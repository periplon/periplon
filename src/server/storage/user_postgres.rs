// PostgreSQL user storage implementation

#[cfg(feature = "server")]
use async_trait::async_trait;
#[cfg(feature = "server")]
use chrono::Utc;
#[cfg(feature = "server")]
use sqlx::{PgPool, Row};
#[cfg(feature = "server")]
use uuid::Uuid;

#[cfg(feature = "server")]
use super::traits::StorageError;
#[cfg(feature = "server")]
use super::user_storage::{Result, User, UserFilter, UserStorage};

/// PostgreSQL-based user storage
/// Uses the `users` table from the database schema
#[cfg(feature = "server")]
pub struct PostgresUserStorage {
    pool: PgPool,
}

#[cfg(feature = "server")]
impl PostgresUserStorage {
    pub async fn new(database_url: &str) -> Result<Self> {
        let pool = PgPool::connect(database_url)
            .await
            .map_err(|e| StorageError::DatabaseError(e.to_string()))?;

        Ok(Self { pool })
    }

    pub fn from_pool(pool: PgPool) -> Self {
        Self { pool }
    }
}

#[cfg(feature = "server")]
#[async_trait]
impl UserStorage for PostgresUserStorage {
    async fn create_user(&self, user: &User) -> Result<Uuid> {
        // Check if email exists
        if self.email_exists(&user.email).await? {
            return Err(StorageError::AlreadyExists(format!(
                "User with email {} already exists",
                user.email
            )));
        }

        sqlx::query(
            r#"
            INSERT INTO users (
                id, email, name, password_hash, is_active, email_verified,
                created_at, updated_at
            )
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8)
            "#,
        )
        .bind(user.id)
        .bind(&user.email)
        .bind(&user.name)
        .bind(&user.password_hash)
        .bind(user.is_active)
        .bind(user.email_verified)
        .bind(user.created_at)
        .bind(user.updated_at)
        .execute(&self.pool)
        .await
        .map_err(|e| StorageError::DatabaseError(e.to_string()))?;

        // Add roles
        for role in &user.roles {
            self.add_role(user.id, role).await?;
        }

        Ok(user.id)
    }

    async fn get_user(&self, id: Uuid) -> Result<Option<User>> {
        let row = sqlx::query(
            r#"
            SELECT id, email, name, password_hash, is_active, email_verified,
                   created_at, updated_at, last_login_at
            FROM users
            WHERE id = $1
            "#,
        )
        .bind(id)
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| StorageError::DatabaseError(e.to_string()))?;

        if let Some(row) = row {
            // Get user roles
            let roles = self.get_user_roles(id).await?;

            Ok(Some(User {
                id: row.get("id"),
                email: row.get("email"),
                name: row.get("name"),
                password_hash: row.get("password_hash"),
                roles,
                is_active: row.get("is_active"),
                email_verified: row.get("email_verified"),
                created_at: row.get("created_at"),
                updated_at: row.get("updated_at"),
                last_login_at: row.get("last_login_at"),
            }))
        } else {
            Ok(None)
        }
    }

    async fn get_user_by_email(&self, email: &str) -> Result<Option<User>> {
        let row = sqlx::query(
            r#"
            SELECT id, email, name, password_hash, is_active, email_verified,
                   created_at, updated_at, last_login_at
            FROM users
            WHERE email = $1
            "#,
        )
        .bind(email)
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| StorageError::DatabaseError(e.to_string()))?;

        if let Some(row) = row {
            let user_id: Uuid = row.get("id");
            let roles = self.get_user_roles(user_id).await?;

            Ok(Some(User {
                id: user_id,
                email: row.get("email"),
                name: row.get("name"),
                password_hash: row.get("password_hash"),
                roles,
                is_active: row.get("is_active"),
                email_verified: row.get("email_verified"),
                created_at: row.get("created_at"),
                updated_at: row.get("updated_at"),
                last_login_at: row.get("last_login_at"),
            }))
        } else {
            Ok(None)
        }
    }

    async fn update_user(&self, id: Uuid, user: &User) -> Result<()> {
        let result = sqlx::query(
            r#"
            UPDATE users
            SET email = $2, name = $3, password_hash = $4, is_active = $5,
                email_verified = $6, updated_at = $7
            WHERE id = $1
            "#,
        )
        .bind(id)
        .bind(&user.email)
        .bind(&user.name)
        .bind(&user.password_hash)
        .bind(user.is_active)
        .bind(user.email_verified)
        .bind(Utc::now())
        .execute(&self.pool)
        .await
        .map_err(|e| StorageError::DatabaseError(e.to_string()))?;

        if result.rows_affected() == 0 {
            return Err(StorageError::NotFound(format!("User {} not found", id)));
        }

        // Update roles: remove all and add new ones
        sqlx::query("DELETE FROM user_roles WHERE user_id = $1")
            .bind(id)
            .execute(&self.pool)
            .await
            .map_err(|e| StorageError::DatabaseError(e.to_string()))?;

        for role in &user.roles {
            self.add_role(id, role).await?;
        }

        Ok(())
    }

    async fn delete_user(&self, id: Uuid) -> Result<()> {
        let result = sqlx::query("DELETE FROM users WHERE id = $1")
            .bind(id)
            .execute(&self.pool)
            .await
            .map_err(|e| StorageError::DatabaseError(e.to_string()))?;

        if result.rows_affected() == 0 {
            return Err(StorageError::NotFound(format!("User {} not found", id)));
        }

        Ok(())
    }

    async fn list_users(&self, filter: &UserFilter) -> Result<Vec<User>> {
        let limit = filter.limit.unwrap_or(100) as i64;
        let offset = filter.offset.unwrap_or(0) as i64;

        let mut query_str = String::from(
            r#"
            SELECT DISTINCT u.id, u.email, u.name, u.password_hash, u.is_active,
                   u.email_verified, u.created_at, u.updated_at, u.last_login_at
            FROM users u
            LEFT JOIN user_roles ur ON u.id = ur.user_id
            LEFT JOIN roles r ON ur.role_id = r.id
            WHERE 1=1
            "#,
        );

        if let Some(ref email) = filter.email {
            query_str.push_str(&format!(" AND u.email ILIKE '%{}%'", email));
        }

        if let Some(is_active) = filter.is_active {
            query_str.push_str(&format!(" AND u.is_active = {}", is_active));
        }

        if let Some(ref role) = filter.role {
            query_str.push_str(&format!(" AND r.name = '{}'", role));
        }

        query_str.push_str(" ORDER BY u.created_at DESC");
        query_str.push_str(&format!(" LIMIT {} OFFSET {}", limit, offset));

        let rows = sqlx::query(&query_str)
            .fetch_all(&self.pool)
            .await
            .map_err(|e| StorageError::DatabaseError(e.to_string()))?;

        let mut users = Vec::new();
        for row in rows {
            let user_id: Uuid = row.get("id");
            let roles = self.get_user_roles(user_id).await?;

            users.push(User {
                id: user_id,
                email: row.get("email"),
                name: row.get("name"),
                password_hash: row.get("password_hash"),
                roles,
                is_active: row.get("is_active"),
                email_verified: row.get("email_verified"),
                created_at: row.get("created_at"),
                updated_at: row.get("updated_at"),
                last_login_at: row.get("last_login_at"),
            });
        }

        Ok(users)
    }

    async fn update_password(&self, id: Uuid, password_hash: &str) -> Result<()> {
        let result =
            sqlx::query("UPDATE users SET password_hash = $2, updated_at = NOW() WHERE id = $1")
                .bind(id)
                .bind(password_hash)
                .execute(&self.pool)
                .await
                .map_err(|e| StorageError::DatabaseError(e.to_string()))?;

        if result.rows_affected() == 0 {
            return Err(StorageError::NotFound(format!("User {} not found", id)));
        }

        Ok(())
    }

    async fn update_last_login(&self, id: Uuid) -> Result<()> {
        let result =
            sqlx::query("UPDATE users SET last_login_at = NOW(), updated_at = NOW() WHERE id = $1")
                .bind(id)
                .execute(&self.pool)
                .await
                .map_err(|e| StorageError::DatabaseError(e.to_string()))?;

        if result.rows_affected() == 0 {
            return Err(StorageError::NotFound(format!("User {} not found", id)));
        }

        Ok(())
    }

    async fn verify_email(&self, id: Uuid) -> Result<()> {
        let result =
            sqlx::query("UPDATE users SET email_verified = true, updated_at = NOW() WHERE id = $1")
                .bind(id)
                .execute(&self.pool)
                .await
                .map_err(|e| StorageError::DatabaseError(e.to_string()))?;

        if result.rows_affected() == 0 {
            return Err(StorageError::NotFound(format!("User {} not found", id)));
        }

        Ok(())
    }

    async fn add_role(&self, id: Uuid, role: &str) -> Result<()> {
        // Get or create role
        let role_id = self.get_or_create_role(role).await?;

        // Add user_role relationship
        sqlx::query(
            "INSERT INTO user_roles (user_id, role_id) VALUES ($1, $2) ON CONFLICT DO NOTHING",
        )
        .bind(id)
        .bind(role_id)
        .execute(&self.pool)
        .await
        .map_err(|e| StorageError::DatabaseError(e.to_string()))?;

        Ok(())
    }

    async fn remove_role(&self, id: Uuid, role: &str) -> Result<()> {
        // Get role ID
        let role_id = match self.get_role_id(role).await? {
            Some(id) => id,
            None => return Ok(()), // Role doesn't exist, nothing to remove
        };

        sqlx::query("DELETE FROM user_roles WHERE user_id = $1 AND role_id = $2")
            .bind(id)
            .bind(role_id)
            .execute(&self.pool)
            .await
            .map_err(|e| StorageError::DatabaseError(e.to_string()))?;

        Ok(())
    }

    async fn has_role(&self, id: Uuid, role: &str) -> Result<bool> {
        let role_id = match self.get_role_id(role).await? {
            Some(id) => id,
            None => return Ok(false),
        };

        let row = sqlx::query(
            "SELECT EXISTS(SELECT 1 FROM user_roles WHERE user_id = $1 AND role_id = $2) as exists",
        )
        .bind(id)
        .bind(role_id)
        .fetch_one(&self.pool)
        .await
        .map_err(|e| StorageError::DatabaseError(e.to_string()))?;

        Ok(row.get("exists"))
    }
}

#[cfg(feature = "server")]
impl PostgresUserStorage {
    /// Helper function to get user roles
    async fn get_user_roles(&self, user_id: Uuid) -> Result<Vec<String>> {
        let rows = sqlx::query(
            r#"
            SELECT r.name
            FROM roles r
            JOIN user_roles ur ON r.id = ur.role_id
            WHERE ur.user_id = $1
            "#,
        )
        .bind(user_id)
        .fetch_all(&self.pool)
        .await
        .map_err(|e| StorageError::DatabaseError(e.to_string()))?;

        Ok(rows.into_iter().map(|row| row.get("name")).collect())
    }

    /// Helper function to get role ID by name
    async fn get_role_id(&self, role_name: &str) -> Result<Option<Uuid>> {
        let row = sqlx::query("SELECT id FROM roles WHERE name = $1")
            .bind(role_name)
            .fetch_optional(&self.pool)
            .await
            .map_err(|e| StorageError::DatabaseError(e.to_string()))?;

        Ok(row.map(|r| r.get("id")))
    }

    /// Helper function to get or create a role
    async fn get_or_create_role(&self, role_name: &str) -> Result<Uuid> {
        // Try to get existing role
        if let Some(role_id) = self.get_role_id(role_name).await? {
            return Ok(role_id);
        }

        // Create new role
        let role_id = Uuid::new_v4();
        sqlx::query(
            "INSERT INTO roles (id, name, description, created_at, updated_at) VALUES ($1, $2, $3, NOW(), NOW())"
        )
        .bind(role_id)
        .bind(role_name)
        .bind(format!("Auto-created role: {}", role_name))
        .execute(&self.pool)
        .await
        .map_err(|e| StorageError::DatabaseError(e.to_string()))?;

        Ok(role_id)
    }
}

#[cfg(all(test, feature = "server"))]
mod tests {

    #[tokio::test]
    #[ignore] // Requires PostgreSQL database
    async fn test_postgres_user_storage() {
        // This test requires a running PostgreSQL instance
        // Run with: cargo test --features server -- --ignored
    }
}
