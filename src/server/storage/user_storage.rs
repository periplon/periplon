// User storage trait for authentication

#[cfg(feature = "server")]
use async_trait::async_trait;
#[cfg(feature = "server")]
use chrono::{DateTime, Utc};
#[cfg(feature = "server")]
use serde::{Deserialize, Serialize};
#[cfg(feature = "server")]
use uuid::Uuid;

#[cfg(feature = "server")]
use super::traits::StorageError;

/// User model for authentication and authorization
#[cfg(feature = "server")]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct User {
    pub id: Uuid,
    pub email: String,
    pub name: String,
    pub password_hash: String,
    pub roles: Vec<String>,
    pub is_active: bool,
    pub email_verified: bool,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub last_login_at: Option<DateTime<Utc>>,
}

/// User filter for querying
#[cfg(feature = "server")]
#[derive(Debug, Clone, Default)]
pub struct UserFilter {
    pub email: Option<String>,
    pub is_active: Option<bool>,
    pub role: Option<String>,
    pub limit: Option<usize>,
    pub offset: Option<usize>,
}

/// Result type for user storage operations
#[cfg(feature = "server")]
pub type Result<T> = std::result::Result<T, StorageError>;

/// User storage trait
/// Provides pluggable storage for user authentication and profiles
#[cfg(feature = "server")]
#[async_trait]
pub trait UserStorage: Send + Sync {
    /// Create a new user
    async fn create_user(&self, user: &User) -> Result<Uuid>;

    /// Get user by ID
    async fn get_user(&self, id: Uuid) -> Result<Option<User>>;

    /// Get user by email
    async fn get_user_by_email(&self, email: &str) -> Result<Option<User>>;

    /// Update user
    async fn update_user(&self, id: Uuid, user: &User) -> Result<()>;

    /// Delete user
    async fn delete_user(&self, id: Uuid) -> Result<()>;

    /// List users with filter
    async fn list_users(&self, filter: &UserFilter) -> Result<Vec<User>>;

    /// Update password hash
    async fn update_password(&self, id: Uuid, password_hash: &str) -> Result<()>;

    /// Update last login time
    async fn update_last_login(&self, id: Uuid) -> Result<()>;

    /// Check if email exists
    async fn email_exists(&self, email: &str) -> Result<bool> {
        Ok(self.get_user_by_email(email).await?.is_some())
    }

    /// Verify user email
    async fn verify_email(&self, id: Uuid) -> Result<()>;

    /// Add role to user
    async fn add_role(&self, id: Uuid, role: &str) -> Result<()>;

    /// Remove role from user
    async fn remove_role(&self, id: Uuid, role: &str) -> Result<()>;

    /// Check if user has role
    async fn has_role(&self, id: Uuid, role: &str) -> Result<bool>;
}

/// Password hashing utilities
#[cfg(feature = "server")]
pub mod password {
    use argon2::{
        password_hash::{
            rand_core::OsRng, PasswordHash, PasswordHasher, PasswordVerifier, SaltString,
        },
        Argon2,
    };

    /// Hash a password using Argon2
    pub fn hash_password(password: &str) -> Result<String, argon2::password_hash::Error> {
        let salt = SaltString::generate(&mut OsRng);
        let argon2 = Argon2::default();
        let password_hash = argon2.hash_password(password.as_bytes(), &salt)?;
        Ok(password_hash.to_string())
    }

    /// Verify a password against a hash
    pub fn verify_password(
        password: &str,
        hash: &str,
    ) -> Result<bool, argon2::password_hash::Error> {
        let parsed_hash = PasswordHash::new(hash)?;
        let argon2 = Argon2::default();
        Ok(argon2
            .verify_password(password.as_bytes(), &parsed_hash)
            .is_ok())
    }

    #[cfg(test)]
    mod tests {
        use super::*;

        #[test]
        fn test_password_hashing() {
            let password = "my_secure_password_123";
            let hash = hash_password(password).unwrap();

            assert!(verify_password(password, &hash).unwrap());
            assert!(!verify_password("wrong_password", &hash).unwrap());
        }
    }
}
