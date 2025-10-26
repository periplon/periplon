// S3 user storage implementation

#[cfg(feature = "server")]
use async_trait::async_trait;
#[cfg(feature = "server")]
use aws_sdk_s3::{primitives::ByteStream, Client as S3Client};
#[cfg(feature = "server")]
use chrono::Utc;
#[cfg(feature = "server")]
use uuid::Uuid;

#[cfg(feature = "server")]
use super::traits::StorageError;
#[cfg(feature = "server")]
use super::user_storage::{Result, User, UserFilter, UserStorage};

/// S3-based user storage
/// Stores users as JSON objects in S3 with a directory-like structure:
/// - users/by-id/{user_id}.json
/// - users/by-email/{email_hash}.json
#[cfg(feature = "server")]
pub struct S3UserStorage {
    client: S3Client,
    bucket: String,
    prefix: String,
}

#[cfg(feature = "server")]
impl S3UserStorage {
    pub async fn new(bucket: String, prefix: Option<String>) -> Result<Self> {
        let config = aws_config::defaults(aws_config::BehaviorVersion::latest())
            .load()
            .await;
        let client = S3Client::new(&config);

        Ok(Self {
            client,
            bucket,
            prefix: prefix.unwrap_or_else(|| "users".to_string()),
        })
    }

    pub fn from_client(client: S3Client, bucket: String, prefix: Option<String>) -> Self {
        Self {
            client,
            bucket,
            prefix: prefix.unwrap_or_else(|| "users".to_string()),
        }
    }

    fn user_key(&self, id: Uuid) -> String {
        format!("{}/by-id/{}.json", self.prefix, id)
    }

    fn email_key(&self, email: &str) -> String {
        // Sanitize email for S3 key
        let safe_email = email.to_lowercase().replace('@', "_at_").replace('.', "_");
        format!("{}/by-email/{}.json", self.prefix, safe_email)
    }

    async fn save_user(&self, user: &User) -> Result<()> {
        let json = serde_json::to_string_pretty(user)
            .map_err(|e| StorageError::SerializationError(e.to_string()))?;

        let user_key = self.user_key(user.id);
        let email_key = self.email_key(&user.email);

        // Save to by-id path
        self.client
            .put_object()
            .bucket(&self.bucket)
            .key(&user_key)
            .body(ByteStream::from(json.clone().into_bytes()))
            .content_type("application/json")
            .send()
            .await
            .map_err(|e| StorageError::IoError(format!("S3 put error: {}", e)))?;

        // Save to by-email path (copy for email lookup)
        self.client
            .put_object()
            .bucket(&self.bucket)
            .key(&email_key)
            .body(ByteStream::from(json.into_bytes()))
            .content_type("application/json")
            .send()
            .await
            .map_err(|e| StorageError::IoError(format!("S3 put error: {}", e)))?;

        Ok(())
    }

    async fn load_user(&self, key: String) -> Result<Option<User>> {
        match self
            .client
            .get_object()
            .bucket(&self.bucket)
            .key(&key)
            .send()
            .await
        {
            Ok(output) => {
                let bytes = output
                    .body
                    .collect()
                    .await
                    .map_err(|e| StorageError::IoError(format!("S3 read error: {}", e)))?
                    .into_bytes();

                let user: User = serde_json::from_slice(&bytes)
                    .map_err(|e| StorageError::SerializationError(e.to_string()))?;

                Ok(Some(user))
            }
            Err(e) => {
                // Check if it's a "not found" error
                if e.to_string().contains("NoSuchKey") {
                    Ok(None)
                } else {
                    Err(StorageError::IoError(format!("S3 get error: {}", e)))
                }
            }
        }
    }

    async fn delete_key(&self, key: &str) -> Result<()> {
        self.client
            .delete_object()
            .bucket(&self.bucket)
            .key(key)
            .send()
            .await
            .map_err(|e| StorageError::IoError(format!("S3 delete error: {}", e)))?;

        Ok(())
    }
}

#[cfg(feature = "server")]
#[async_trait]
impl UserStorage for S3UserStorage {
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
        let key = self.user_key(id);
        self.load_user(key).await
    }

    async fn get_user_by_email(&self, email: &str) -> Result<Option<User>> {
        let key = self.email_key(email);
        self.load_user(key).await
    }

    async fn update_user(&self, id: Uuid, user: &User) -> Result<()> {
        // Check if user exists
        if self.get_user(id).await?.is_none() {
            return Err(StorageError::NotFound(format!("User {} not found", id)));
        }

        // If email changed, remove old email lookup
        if let Some(old_user) = self.get_user(id).await? {
            if old_user.email != user.email {
                let old_email_key = self.email_key(&old_user.email);
                let _ = self.delete_key(&old_email_key).await; // Ignore errors
            }
        }

        self.save_user(user).await
    }

    async fn delete_user(&self, id: Uuid) -> Result<()> {
        let user = self
            .get_user(id)
            .await?
            .ok_or_else(|| StorageError::NotFound(format!("User {} not found", id)))?;

        let user_key = self.user_key(id);
        let email_key = self.email_key(&user.email);

        self.delete_key(&user_key).await?;
        let _ = self.delete_key(&email_key).await; // Ignore errors

        Ok(())
    }

    async fn list_users(&self, filter: &UserFilter) -> Result<Vec<User>> {
        let prefix = format!("{}/by-id/", self.prefix);

        let mut users = Vec::new();
        let mut continuation_token: Option<String> = None;

        // Paginate through S3 results
        loop {
            let mut request = self
                .client
                .list_objects_v2()
                .bucket(&self.bucket)
                .prefix(&prefix);

            if let Some(token) = continuation_token {
                request = request.continuation_token(token);
            }

            let response = request
                .send()
                .await
                .map_err(|e| StorageError::IoError(format!("S3 list error: {}", e)))?;

            if let Some(contents) = response.contents {
                for object in contents {
                    if let Some(key) = object.key {
                        if key.ends_with(".json") {
                            if let Some(user) = self.load_user(key).await? {
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
                }
            }

            // Check if there are more results
            if response.is_truncated == Some(true) {
                continuation_token = response.next_continuation_token;
            } else {
                break;
            }
        }

        // Sort by created_at (newest first) to match other implementations
        users.sort_by(|a, b| b.created_at.cmp(&a.created_at));

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

    #[tokio::test]
    #[ignore] // Requires AWS credentials and S3 bucket
    async fn test_s3_user_storage() {
        // This test requires AWS credentials configured and a test bucket
        // Run with: cargo test --features server -- --ignored

        let bucket = std::env::var("TEST_S3_BUCKET").unwrap_or_else(|_| "test-bucket".to_string());

        let storage = S3UserStorage::new(bucket, Some("test-users".to_string()))
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
