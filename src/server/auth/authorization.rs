// Authorization service for permission checking

#[cfg(feature = "server")]
use async_trait::async_trait;
#[cfg(feature = "server")]
use sqlx::PgPool;
#[cfg(feature = "server")]
use std::sync::Arc;
#[cfg(feature = "server")]
use uuid::Uuid;

/// Permission check result
#[cfg(feature = "server")]
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PermissionResult {
    Allowed,
    Denied,
}

/// Authorization service trait
#[cfg(feature = "server")]
#[async_trait]
pub trait AuthorizationService: Send + Sync {
    /// Check if user has a specific permission
    async fn has_permission(&self, user_id: &str, permission: &str) -> Result<bool, String>;

    /// Check if user has role
    async fn has_role(&self, user_id: &str, role_name: &str) -> Result<bool, String>;

    /// Check resource-level permission (e.g., workflow ownership)
    async fn can_access_resource(
        &self,
        user_id: &str,
        resource_type: &str,
        resource_id: Uuid,
        action: &str,
    ) -> Result<bool, String>;

    /// Get user's permissions
    async fn get_user_permissions(&self, user_id: &str) -> Result<Vec<String>, String>;
}

/// PostgreSQL-based authorization service
#[cfg(feature = "server")]
pub struct PostgresAuthorizationService {
    pool: Arc<PgPool>,
}

#[cfg(feature = "server")]
impl PostgresAuthorizationService {
    pub fn new(pool: Arc<PgPool>) -> Self {
        Self { pool }
    }
}

#[cfg(feature = "server")]
#[async_trait]
impl AuthorizationService for PostgresAuthorizationService {
    async fn has_permission(&self, user_id: &str, permission: &str) -> Result<bool, String> {
        // Parse user_id as UUID
        let user_uuid = Uuid::parse_str(user_id).map_err(|e| format!("Invalid user ID: {}", e))?;

        // Query to check if user has permission through their roles
        let result = sqlx::query_scalar::<_, bool>(
            r#"
            SELECT EXISTS(
                SELECT 1
                FROM user_roles ur
                JOIN role_permissions rp ON ur.role_id = rp.role_id
                JOIN permissions p ON rp.permission_id = p.id
                WHERE ur.user_id = $1
                  AND p.name = $2
                  AND (ur.expires_at IS NULL OR ur.expires_at > NOW())
            )
            "#,
        )
        .bind(user_uuid)
        .bind(permission)
        .fetch_one(self.pool.as_ref())
        .await
        .map_err(|e| format!("Database error: {}", e))?;

        Ok(result)
    }

    async fn has_role(&self, user_id: &str, role_name: &str) -> Result<bool, String> {
        let user_uuid = Uuid::parse_str(user_id).map_err(|e| format!("Invalid user ID: {}", e))?;

        let result = sqlx::query_scalar::<_, bool>(
            r#"
            SELECT EXISTS(
                SELECT 1
                FROM user_roles ur
                JOIN roles r ON ur.role_id = r.id
                WHERE ur.user_id = $1
                  AND r.name = $2
                  AND (ur.expires_at IS NULL OR ur.expires_at > NOW())
            )
            "#,
        )
        .bind(user_uuid)
        .bind(role_name)
        .fetch_one(self.pool.as_ref())
        .await
        .map_err(|e| format!("Database error: {}", e))?;

        Ok(result)
    }

    async fn can_access_resource(
        &self,
        user_id: &str,
        resource_type: &str,
        resource_id: Uuid,
        action: &str,
    ) -> Result<bool, String> {
        let user_uuid = Uuid::parse_str(user_id).map_err(|e| format!("Invalid user ID: {}", e))?;

        // Check resource-specific permissions based on type
        match resource_type {
            "workflow" => {
                // Check workflow_permissions table
                let result = sqlx::query_scalar::<_, bool>(
                    r#"
                    SELECT EXISTS(
                        SELECT 1
                        FROM user_workflow_permissions
                        WHERE workflow_id = $1
                          AND user_id = $2
                          AND permission_level >= CASE
                            WHEN $3 = 'read' THEN 1
                            WHEN $3 = 'write' THEN 2
                            WHEN $3 = 'admin' THEN 3
                            WHEN $3 = 'owner' THEN 4
                            ELSE 0
                          END
                    )
                    "#,
                )
                .bind(resource_id)
                .bind(user_uuid)
                .bind(action)
                .fetch_one(self.pool.as_ref())
                .await
                .map_err(|e| format!("Database error: {}", e))?;

                Ok(result)
            }
            "execution" => {
                // Check execution_permissions table
                let result = sqlx::query_scalar::<_, bool>(
                    r#"
                    SELECT EXISTS(
                        SELECT 1
                        FROM execution_permissions
                        WHERE execution_id = $1
                          AND subject_type = 'user'
                          AND subject_id = $2
                    )
                    "#,
                )
                .bind(resource_id)
                .bind(user_uuid)
                .fetch_one(self.pool.as_ref())
                .await
                .map_err(|e| format!("Database error: {}", e))?;

                Ok(result)
            }
            _ => {
                // For other resources, fall back to general permission check
                let permission_name = format!("{}.{}", resource_type, action);
                self.has_permission(user_id, &permission_name).await
            }
        }
    }

    async fn get_user_permissions(&self, user_id: &str) -> Result<Vec<String>, String> {
        let user_uuid = Uuid::parse_str(user_id).map_err(|e| format!("Invalid user ID: {}", e))?;

        let permissions = sqlx::query_scalar::<_, String>(
            r#"
            SELECT DISTINCT p.name
            FROM user_roles ur
            JOIN role_permissions rp ON ur.role_id = rp.role_id
            JOIN permissions p ON rp.permission_id = p.id
            WHERE ur.user_id = $1
              AND (ur.expires_at IS NULL OR ur.expires_at > NOW())
            ORDER BY p.name
            "#,
        )
        .bind(user_uuid)
        .fetch_all(self.pool.as_ref())
        .await
        .map_err(|e| format!("Database error: {}", e))?;

        Ok(permissions)
    }
}

/// Helper function to check permission with common error handling
#[cfg(feature = "server")]
pub async fn check_permission(
    auth_service: &Arc<dyn AuthorizationService>,
    user_id: &str,
    permission: &str,
) -> Result<(), String> {
    match auth_service.has_permission(user_id, permission).await {
        Ok(true) => Ok(()),
        Ok(false) => Err(format!("Permission denied: {}", permission)),
        Err(e) => Err(format!("Authorization error: {}", e)),
    }
}

/// Helper function to check resource access
#[cfg(feature = "server")]
pub async fn check_resource_access(
    auth_service: &Arc<dyn AuthorizationService>,
    user_id: &str,
    resource_type: &str,
    resource_id: Uuid,
    action: &str,
) -> Result<(), String> {
    match auth_service
        .can_access_resource(user_id, resource_type, resource_id, action)
        .await
    {
        Ok(true) => Ok(()),
        Ok(false) => Err(format!(
            "Access denied to {} {}",
            resource_type, resource_id
        )),
        Err(e) => Err(format!("Authorization error: {}", e)),
    }
}
