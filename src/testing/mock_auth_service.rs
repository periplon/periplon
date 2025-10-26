//! Mock Authentication and Authorization Services
//!
//! Provides in-memory mock implementations for testing authentication
//! and authorization flows without requiring database connections.

#[cfg(feature = "server")]
use async_trait::async_trait;
#[cfg(feature = "server")]
use chrono::Utc;
#[cfg(feature = "server")]
use std::collections::HashMap;
#[cfg(feature = "server")]
use std::sync::{Arc, Mutex};
#[cfg(feature = "server")]
use uuid::Uuid;

#[cfg(feature = "server")]
use crate::server::auth::authorization::AuthorizationService;
#[cfg(feature = "server")]
use crate::server::storage::traits::StorageError;
#[cfg(feature = "server")]
use crate::server::storage::user_storage::{Result, User, UserStorage};

/// Mock user storage for testing authentication
#[cfg(feature = "server")]
pub struct MockUserStorage {
    state: Arc<Mutex<UserStorageState>>,
}

#[cfg(feature = "server")]
struct UserStorageState {
    users: HashMap<Uuid, User>,
    users_by_email: HashMap<String, Uuid>,
    should_fail: bool,
}

#[cfg(feature = "server")]
impl MockUserStorage {
    pub fn new() -> Self {
        Self {
            state: Arc::new(Mutex::new(UserStorageState {
                users: HashMap::new(),
                users_by_email: HashMap::new(),
                should_fail: false,
            })),
        }
    }

    /// Add a test user
    pub fn add_user(&self, user: User) {
        let mut state = self.state.lock().unwrap();
        state.users_by_email.insert(user.email.clone(), user.id);
        state.users.insert(user.id, user);
    }

    /// Enable failure mode for testing error scenarios
    pub fn fail_operations(&self) {
        let mut state = self.state.lock().unwrap();
        state.should_fail = true;
    }

    /// Get user count
    pub fn user_count(&self) -> usize {
        let state = self.state.lock().unwrap();
        state.users.len()
    }

    /// Clear all users
    pub fn clear(&self) {
        let mut state = self.state.lock().unwrap();
        state.users.clear();
        state.users_by_email.clear();
    }
}

#[cfg(feature = "server")]
impl Default for MockUserStorage {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(feature = "server")]
#[async_trait]
impl UserStorage for MockUserStorage {
    async fn create_user(&self, user: &User) -> Result<Uuid> {
        let mut state = self.state.lock().unwrap();

        if state.should_fail {
            return Err(StorageError::IoError("Simulated failure".to_string()));
        }

        if state.users_by_email.contains_key(&user.email) {
            return Err(StorageError::AlreadyExists(
                "User with this email already exists".to_string(),
            ));
        }

        state.users_by_email.insert(user.email.clone(), user.id);
        state.users.insert(user.id, user.clone());
        Ok(user.id)
    }

    async fn get_user(&self, user_id: Uuid) -> Result<Option<User>> {
        let state = self.state.lock().unwrap();

        if state.should_fail {
            return Err(StorageError::IoError("Simulated failure".to_string()));
        }

        Ok(state.users.get(&user_id).cloned())
    }

    async fn get_user_by_email(&self, email: &str) -> Result<Option<User>> {
        let state = self.state.lock().unwrap();

        if state.should_fail {
            return Err(StorageError::IoError("Simulated failure".to_string()));
        }

        if let Some(user_id) = state.users_by_email.get(email) {
            Ok(state.users.get(user_id).cloned())
        } else {
            Ok(None)
        }
    }

    async fn update_user(&self, user_id: Uuid, user: &User) -> Result<()> {
        let mut state = self.state.lock().unwrap();

        if state.should_fail {
            return Err(StorageError::IoError("Simulated failure".to_string()));
        }

        if !state.users.contains_key(&user_id) {
            return Err(StorageError::NotFound("User not found".to_string()));
        }

        state.users.insert(user_id, user.clone());
        Ok(())
    }

    async fn delete_user(&self, user_id: Uuid) -> Result<()> {
        let mut state = self.state.lock().unwrap();

        if state.should_fail {
            return Err(StorageError::IoError("Simulated failure".to_string()));
        }

        if let Some(user) = state.users.remove(&user_id) {
            state.users_by_email.remove(&user.email);
            Ok(())
        } else {
            Err(StorageError::NotFound("User not found".to_string()))
        }
    }

    async fn update_last_login(&self, user_id: Uuid) -> Result<()> {
        let mut state = self.state.lock().unwrap();

        if state.should_fail {
            return Err(StorageError::IoError("Simulated failure".to_string()));
        }

        if let Some(user) = state.users.get_mut(&user_id) {
            user.last_login_at = Some(Utc::now());
            Ok(())
        } else {
            Err(StorageError::NotFound("User not found".to_string()))
        }
    }

    async fn list_users(
        &self,
        _filter: &crate::server::storage::user_storage::UserFilter,
    ) -> Result<Vec<User>> {
        let state = self.state.lock().unwrap();

        if state.should_fail {
            return Err(StorageError::IoError("Simulated failure".to_string()));
        }

        Ok(state.users.values().cloned().collect())
    }

    async fn update_password(&self, user_id: Uuid, password_hash: &str) -> Result<()> {
        let mut state = self.state.lock().unwrap();

        if state.should_fail {
            return Err(StorageError::IoError("Simulated failure".to_string()));
        }

        if let Some(user) = state.users.get_mut(&user_id) {
            user.password_hash = password_hash.to_string();
            Ok(())
        } else {
            Err(StorageError::NotFound("User not found".to_string()))
        }
    }

    async fn verify_email(&self, user_id: Uuid) -> Result<()> {
        let mut state = self.state.lock().unwrap();

        if state.should_fail {
            return Err(StorageError::IoError("Simulated failure".to_string()));
        }

        if let Some(user) = state.users.get_mut(&user_id) {
            user.email_verified = true;
            Ok(())
        } else {
            Err(StorageError::NotFound("User not found".to_string()))
        }
    }

    async fn add_role(&self, user_id: Uuid, role: &str) -> Result<()> {
        let mut state = self.state.lock().unwrap();

        if state.should_fail {
            return Err(StorageError::IoError("Simulated failure".to_string()));
        }

        if let Some(user) = state.users.get_mut(&user_id) {
            if !user.roles.contains(&role.to_string()) {
                user.roles.push(role.to_string());
            }
            Ok(())
        } else {
            Err(StorageError::NotFound("User not found".to_string()))
        }
    }

    async fn remove_role(&self, user_id: Uuid, role: &str) -> Result<()> {
        let mut state = self.state.lock().unwrap();

        if state.should_fail {
            return Err(StorageError::IoError("Simulated failure".to_string()));
        }

        if let Some(user) = state.users.get_mut(&user_id) {
            user.roles.retain(|r| r != role);
            Ok(())
        } else {
            Err(StorageError::NotFound("User not found".to_string()))
        }
    }

    async fn has_role(&self, user_id: Uuid, role: &str) -> Result<bool> {
        let state = self.state.lock().unwrap();

        if state.should_fail {
            return Err(StorageError::IoError("Simulated failure".to_string()));
        }

        Ok(state
            .users
            .get(&user_id)
            .map(|user| user.roles.contains(&role.to_string()))
            .unwrap_or(false))
    }
}

/// Mock authorization service for testing permissions
#[cfg(feature = "server")]
pub struct MockAuthorizationService {
    state: Arc<Mutex<AuthorizationState>>,
}

#[cfg(feature = "server")]
struct AuthorizationState {
    user_permissions: HashMap<String, Vec<String>>,
    user_roles: HashMap<String, Vec<String>>,
    resource_permissions: HashMap<(String, Uuid), HashMap<String, Vec<String>>>,
    should_fail: bool,
}

#[cfg(feature = "server")]
impl MockAuthorizationService {
    pub fn new() -> Self {
        Self {
            state: Arc::new(Mutex::new(AuthorizationState {
                user_permissions: HashMap::new(),
                user_roles: HashMap::new(),
                resource_permissions: HashMap::new(),
                should_fail: false,
            })),
        }
    }

    /// Grant permission to a user
    pub fn grant_permission(&self, user_id: &str, permission: &str) {
        let mut state = self.state.lock().unwrap();
        state
            .user_permissions
            .entry(user_id.to_string())
            .or_default()
            .push(permission.to_string());
    }

    /// Grant role to a user
    pub fn grant_role(&self, user_id: &str, role: &str) {
        let mut state = self.state.lock().unwrap();
        state
            .user_roles
            .entry(user_id.to_string())
            .or_default()
            .push(role.to_string());
    }

    /// Grant resource access to a user
    pub fn grant_resource_access(
        &self,
        user_id: &str,
        resource_type: &str,
        resource_id: Uuid,
        action: &str,
    ) {
        let mut state = self.state.lock().unwrap();
        let key = (resource_type.to_string(), resource_id);
        state
            .resource_permissions
            .entry(key)
            .or_default()
            .entry(user_id.to_string())
            .or_default()
            .push(action.to_string());
    }

    /// Revoke permission from a user
    pub fn revoke_permission(&self, user_id: &str, permission: &str) {
        let mut state = self.state.lock().unwrap();
        if let Some(perms) = state.user_permissions.get_mut(user_id) {
            perms.retain(|p| p != permission);
        }
    }

    /// Enable failure mode
    pub fn fail_operations(&self) {
        let mut state = self.state.lock().unwrap();
        state.should_fail = true;
    }

    /// Clear all permissions and roles
    pub fn clear(&self) {
        let mut state = self.state.lock().unwrap();
        state.user_permissions.clear();
        state.user_roles.clear();
        state.resource_permissions.clear();
    }
}

#[cfg(feature = "server")]
impl Default for MockAuthorizationService {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(feature = "server")]
#[async_trait]
impl AuthorizationService for MockAuthorizationService {
    async fn has_permission(
        &self,
        user_id: &str,
        permission: &str,
    ) -> std::result::Result<bool, String> {
        let state = self.state.lock().unwrap();

        if state.should_fail {
            return Err("Simulated authorization failure".to_string());
        }

        Ok(state
            .user_permissions
            .get(user_id)
            .map(|perms| perms.contains(&permission.to_string()))
            .unwrap_or(false))
    }

    async fn has_role(&self, user_id: &str, role_name: &str) -> std::result::Result<bool, String> {
        let state = self.state.lock().unwrap();

        if state.should_fail {
            return Err("Simulated authorization failure".to_string());
        }

        Ok(state
            .user_roles
            .get(user_id)
            .map(|roles| roles.contains(&role_name.to_string()))
            .unwrap_or(false))
    }

    async fn can_access_resource(
        &self,
        user_id: &str,
        resource_type: &str,
        resource_id: Uuid,
        action: &str,
    ) -> std::result::Result<bool, String> {
        let state = self.state.lock().unwrap();

        if state.should_fail {
            return Err("Simulated authorization failure".to_string());
        }

        let key = (resource_type.to_string(), resource_id);
        Ok(state
            .resource_permissions
            .get(&key)
            .and_then(|user_perms| user_perms.get(user_id))
            .map(|actions| actions.contains(&action.to_string()))
            .unwrap_or(false))
    }

    async fn get_user_permissions(
        &self,
        user_id: &str,
    ) -> std::result::Result<Vec<String>, String> {
        let state = self.state.lock().unwrap();

        if state.should_fail {
            return Err("Simulated authorization failure".to_string());
        }

        Ok(state
            .user_permissions
            .get(user_id)
            .cloned()
            .unwrap_or_default())
    }
}

#[cfg(all(test, feature = "server"))]
mod tests {
    use super::*;
    use crate::server::storage::password;

    fn create_test_user(email: &str, roles: Vec<String>) -> User {
        User {
            id: Uuid::new_v4(),
            email: email.to_string(),
            name: "Test User".to_string(),
            password_hash: password::hash_password("password123").unwrap(),
            roles,
            is_active: true,
            email_verified: false,
            created_at: Utc::now(),
            updated_at: Utc::now(),
            last_login_at: None,
        }
    }

    #[tokio::test]
    async fn test_mock_user_storage_create() {
        let storage = MockUserStorage::new();
        let user = create_test_user("test@example.com", vec!["user".to_string()]);

        let user_id = storage.create_user(&user).await.unwrap();
        assert_eq!(user_id, user.id);
        assert_eq!(storage.user_count(), 1);
    }

    #[tokio::test]
    async fn test_mock_user_storage_get() {
        let storage = MockUserStorage::new();
        let user = create_test_user("test@example.com", vec!["user".to_string()]);

        storage.create_user(&user).await.unwrap();

        let retrieved = storage.get_user(user.id).await.unwrap();
        assert!(retrieved.is_some());
        assert_eq!(retrieved.unwrap().email, "test@example.com");
    }

    #[tokio::test]
    async fn test_mock_user_storage_duplicate_email() {
        let storage = MockUserStorage::new();
        let user1 = create_test_user("test@example.com", vec!["user".to_string()]);
        let user2 = create_test_user("test@example.com", vec!["admin".to_string()]);

        storage.create_user(&user1).await.unwrap();
        let result = storage.create_user(&user2).await;

        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_mock_authorization_permissions() {
        let auth = MockAuthorizationService::new();
        let user_id = "user123";

        auth.grant_permission(user_id, "workflows:read");
        auth.grant_permission(user_id, "workflows:write");

        assert!(auth
            .has_permission(user_id, "workflows:read")
            .await
            .unwrap());
        assert!(auth
            .has_permission(user_id, "workflows:write")
            .await
            .unwrap());
        assert!(!auth
            .has_permission(user_id, "workflows:delete")
            .await
            .unwrap());
    }

    #[tokio::test]
    async fn test_mock_authorization_roles() {
        let auth = MockAuthorizationService::new();
        let user_id = "user123";

        auth.grant_role(user_id, "admin");
        auth.grant_role(user_id, "developer");

        assert!(auth.has_role(user_id, "admin").await.unwrap());
        assert!(auth.has_role(user_id, "developer").await.unwrap());
        assert!(!auth.has_role(user_id, "viewer").await.unwrap());
    }

    #[tokio::test]
    async fn test_mock_authorization_resource_access() {
        let auth = MockAuthorizationService::new();
        let user_id = "user123";
        let workflow_id = Uuid::new_v4();

        auth.grant_resource_access(user_id, "workflow", workflow_id, "read");
        auth.grant_resource_access(user_id, "workflow", workflow_id, "write");

        assert!(auth
            .can_access_resource(user_id, "workflow", workflow_id, "read")
            .await
            .unwrap());
        assert!(auth
            .can_access_resource(user_id, "workflow", workflow_id, "write")
            .await
            .unwrap());
        assert!(!auth
            .can_access_resource(user_id, "workflow", workflow_id, "delete")
            .await
            .unwrap());
    }
}
