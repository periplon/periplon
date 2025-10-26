// Filesystem user storage implementation

#[cfg(feature = "server")]
use async_trait::async_trait;
#[cfg(feature = "server")]
use chrono::Utc;
#[cfg(feature = "server")]
use std::path::PathBuf;
#[cfg(feature = "server")]
use tokio::fs;
#[cfg(feature = "server")]
use uuid::Uuid;

#[cfg(feature = "server")]
use super::traits::StorageError;
#[cfg(feature = "server")]
use super::user_storage::{Result, User, UserFilter, UserStorage};

/// Filesystem-based user storage
/// Stores users as JSON files in a directory structure:
/// - users/
///   - by-id/
///     - {user_id}.json
///   - by-email/
///     - {email_hash}.json (symlink to by-id file)
#[cfg(feature = "server")]
pub struct FilesystemUserStorage {
    base_path: PathBuf,
}

#[cfg(feature = "server")]
impl FilesystemUserStorage {
    pub async fn new(base_path: PathBuf) -> Result<Self> {
        let users_dir = base_path.join("users");
        let by_id_dir = users_dir.join("by-id");
        let by_email_dir = users_dir.join("by-email");

        // Create directories
        fs::create_dir_all(&by_id_dir)
            .await
            .map_err(|e| StorageError::IoError(e.to_string()))?;
        fs::create_dir_all(&by_email_dir)
            .await
            .map_err(|e| StorageError::IoError(e.to_string()))?;

        Ok(Self { base_path })
    }

    fn user_path(&self, id: Uuid) -> PathBuf {
        self.base_path
            .join("users")
            .join("by-id")
            .join(format!("{}.json", id))
    }

    fn email_path(&self, email: &str) -> PathBuf {
        // Use email as filename with @ replaced by _at_
        let safe_email = email.to_lowercase().replace('@', "_at_").replace('.', "_");
        self.base_path
            .join("users")
            .join("by-email")
            .join(format!("{}.json", safe_email))
    }

    async fn save_user(&self, user: &User) -> Result<()> {
        let user_path = self.user_path(user.id);
        let email_path = self.email_path(&user.email);

        // Save user JSON
        let json = serde_json::to_string_pretty(user)
            .map_err(|e| StorageError::SerializationError(e.to_string()))?;

        fs::write(&user_path, json)
            .await
            .map_err(|e| StorageError::IoError(e.to_string()))?;

        // Create email lookup (copy of user data for simplicity)
        // In a more advanced implementation, this could be a symlink
        fs::copy(&user_path, &email_path)
            .await
            .map_err(|e| StorageError::IoError(e.to_string()))?;

        Ok(())
    }

    async fn load_user(&self, path: PathBuf) -> Result<Option<User>> {
        if !path.exists() {
            return Ok(None);
        }

        let content = fs::read_to_string(&path)
            .await
            .map_err(|e| StorageError::IoError(e.to_string()))?;

        let user: User = serde_json::from_str(&content)
            .map_err(|e| StorageError::SerializationError(e.to_string()))?;

        Ok(Some(user))
    }
}

#[cfg(feature = "server")]
#[async_trait]
impl UserStorage for FilesystemUserStorage {
    async fn create_user(&self, user: &User) -> Result<Uuid> {
        // Check if email already exists
        if self.email_exists(&user.email).await? {
            return Err(StorageError::AlreadyExists(format!(
                "User with email {} already exists",
                user.email
            )));
        }

        self.save_user(user).await?;
        Ok(user.id)
    }

    async fn get_user(&self, id: Uuid) -> Result<Option<User>> {
        let path = self.user_path(id);
        self.load_user(path).await
    }

    async fn get_user_by_email(&self, email: &str) -> Result<Option<User>> {
        let path = self.email_path(email);
        self.load_user(path).await
    }

    async fn update_user(&self, id: Uuid, user: &User) -> Result<()> {
        // Check if user exists
        if self.get_user(id).await?.is_none() {
            return Err(StorageError::NotFound(format!("User {} not found", id)));
        }

        // If email changed, remove old email lookup
        if let Some(old_user) = self.get_user(id).await? {
            if old_user.email != user.email {
                let old_email_path = self.email_path(&old_user.email);
                let _ = fs::remove_file(old_email_path).await; // Ignore errors
            }
        }

        self.save_user(user).await
    }

    async fn delete_user(&self, id: Uuid) -> Result<()> {
        let user = self
            .get_user(id)
            .await?
            .ok_or_else(|| StorageError::NotFound(format!("User {} not found", id)))?;

        let user_path = self.user_path(id);
        let email_path = self.email_path(&user.email);

        fs::remove_file(&user_path)
            .await
            .map_err(|e| StorageError::IoError(e.to_string()))?;

        let _ = fs::remove_file(&email_path).await; // Ignore errors

        Ok(())
    }

    async fn list_users(&self, filter: &UserFilter) -> Result<Vec<User>> {
        let by_id_dir = self.base_path.join("users").join("by-id");

        let mut entries = fs::read_dir(&by_id_dir)
            .await
            .map_err(|e| StorageError::IoError(e.to_string()))?;

        let mut users = Vec::new();

        while let Some(entry) = entries
            .next_entry()
            .await
            .map_err(|e| StorageError::IoError(e.to_string()))?
        {
            if entry.path().extension().and_then(|s| s.to_str()) == Some("json") {
                if let Some(user) = self.load_user(entry.path()).await? {
                    // Apply filters
                    if let Some(ref email) = filter.email {
                        if !user.email.contains(email) {
                            continue;
                        }
                    }

                    if let Some(is_active) = filter.is_active {
                        if user.is_active != is_active {
                            continue;
                        }
                    }

                    if let Some(ref role) = filter.role {
                        if !user.roles.contains(role) {
                            continue;
                        }
                    }

                    users.push(user);
                }
            }
        }

        // Apply pagination
        let offset = filter.offset.unwrap_or(0);
        let limit = filter.limit.unwrap_or(100);

        Ok(users.into_iter().skip(offset).take(limit).collect())
    }

    async fn update_password(&self, id: Uuid, password_hash: &str) -> Result<()> {
        let mut user = self
            .get_user(id)
            .await?
            .ok_or_else(|| StorageError::NotFound(format!("User {} not found", id)))?;

        user.password_hash = password_hash.to_string();
        user.updated_at = Utc::now();

        self.save_user(&user).await
    }

    async fn update_last_login(&self, id: Uuid) -> Result<()> {
        let mut user = self
            .get_user(id)
            .await?
            .ok_or_else(|| StorageError::NotFound(format!("User {} not found", id)))?;

        user.last_login_at = Some(Utc::now());
        user.updated_at = Utc::now();

        self.save_user(&user).await
    }

    async fn verify_email(&self, id: Uuid) -> Result<()> {
        let mut user = self
            .get_user(id)
            .await?
            .ok_or_else(|| StorageError::NotFound(format!("User {} not found", id)))?;

        user.email_verified = true;
        user.updated_at = Utc::now();

        self.save_user(&user).await
    }

    async fn add_role(&self, id: Uuid, role: &str) -> Result<()> {
        let mut user = self
            .get_user(id)
            .await?
            .ok_or_else(|| StorageError::NotFound(format!("User {} not found", id)))?;

        if !user.roles.contains(&role.to_string()) {
            user.roles.push(role.to_string());
            user.updated_at = Utc::now();
            self.save_user(&user).await?;
        }

        Ok(())
    }

    async fn remove_role(&self, id: Uuid, role: &str) -> Result<()> {
        let mut user = self
            .get_user(id)
            .await?
            .ok_or_else(|| StorageError::NotFound(format!("User {} not found", id)))?;

        user.roles.retain(|r| r != role);
        user.updated_at = Utc::now();

        self.save_user(&user).await
    }

    async fn has_role(&self, id: Uuid, role: &str) -> Result<bool> {
        let user = self
            .get_user(id)
            .await?
            .ok_or_else(|| StorageError::NotFound(format!("User {} not found", id)))?;

        Ok(user.roles.contains(&role.to_string()))
    }
}

#[cfg(all(test, feature = "server"))]
mod tests {
    use super::*;
    use tempfile::tempdir;

    #[tokio::test]
    async fn test_filesystem_user_storage() {
        let dir = tempdir().unwrap();
        let storage = FilesystemUserStorage::new(dir.path().to_path_buf())
            .await
            .unwrap();

        let user = User {
            id: Uuid::new_v4(),
            email: "test@example.com".to_string(),
            name: "Test User".to_string(),
            password_hash: "hash123".to_string(),
            roles: vec!["user".to_string()],
            is_active: true,
            email_verified: false,
            created_at: Utc::now(),
            updated_at: Utc::now(),
            last_login_at: None,
        };

        // Create user
        let id = storage.create_user(&user).await.unwrap();
        assert_eq!(id, user.id);

        // Get by ID
        let retrieved = storage.get_user(id).await.unwrap().unwrap();
        assert_eq!(retrieved.email, user.email);

        // Get by email
        let by_email = storage
            .get_user_by_email("test@example.com")
            .await
            .unwrap()
            .unwrap();
        assert_eq!(by_email.id, user.id);

        // Update password
        storage.update_password(id, "newhash").await.unwrap();
        let updated = storage.get_user(id).await.unwrap().unwrap();
        assert_eq!(updated.password_hash, "newhash");

        // Add role
        storage.add_role(id, "admin").await.unwrap();
        assert!(storage.has_role(id, "admin").await.unwrap());

        // Delete user
        storage.delete_user(id).await.unwrap();
        assert!(storage.get_user(id).await.unwrap().is_none());
    }
}
