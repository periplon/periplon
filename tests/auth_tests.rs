//! Authentication and Authorization Tests
//!
//! Comprehensive tests for JWT tokens, user authentication, role-based
//! access control, and resource-level permissions.

#[cfg(all(test, feature = "server"))]
mod auth_tests {
    use chrono::{Duration, Utc};
    use periplon_sdk::server::auth::authorization::AuthorizationService;
    use periplon_sdk::server::auth::{Claims, JwtManager};
    use periplon_sdk::server::storage::password;
    use periplon_sdk::server::storage::user_storage::{User, UserStorage};
    use periplon_sdk::testing::{MockAuthorizationService, MockUserStorage};
    use std::sync::Arc;
    use uuid::Uuid;

    fn create_test_user(email: &str, password: &str, roles: Vec<String>) -> User {
        User {
            id: Uuid::new_v4(),
            email: email.to_string(),
            name: "Test User".to_string(),
            password_hash: password::hash_password(password).unwrap(),
            roles,
            is_active: true,
            email_verified: false,
            created_at: Utc::now(),
            updated_at: Utc::now(),
            last_login_at: None,
        }
    }

    // ===== JWT Token Tests =====

    #[test]
    fn test_jwt_token_generation() {
        let jwt_manager = JwtManager::new("test_secret_key_123", 24);

        let token = jwt_manager
            .generate_token("user123", "test@example.com", vec!["user".to_string()])
            .expect("Should generate token");

        assert!(!token.is_empty());
        assert!(token.contains('.'), "JWT should have dot separators");
    }

    #[test]
    fn test_jwt_token_validation() {
        let jwt_manager = JwtManager::new("test_secret_key_123", 24);

        let token = jwt_manager
            .generate_token(
                "user123",
                "test@example.com",
                vec!["user".to_string(), "admin".to_string()],
            )
            .unwrap();

        let claims = jwt_manager
            .validate_token(&token)
            .expect("Should validate token");

        assert_eq!(claims.sub, "user123");
        assert_eq!(claims.email, "test@example.com");
        assert_eq!(claims.roles.len(), 2);
        assert!(claims.roles.contains(&"user".to_string()));
        assert!(claims.roles.contains(&"admin".to_string()));
    }

    #[test]
    fn test_jwt_token_expiration() {
        let jwt_manager = JwtManager::new("test_secret_key_123", 1);

        let token = jwt_manager
            .generate_token("user123", "test@example.com", vec!["user".to_string()])
            .unwrap();

        let claims = jwt_manager.validate_token(&token).unwrap();

        // Token should not be expired immediately
        assert!(!jwt_manager.is_token_expired(&claims));

        // Create an expired token manually
        let expired_claims = Claims {
            sub: "user123".to_string(),
            email: "test@example.com".to_string(),
            exp: (Utc::now() - Duration::hours(2)).timestamp(),
            iat: (Utc::now() - Duration::hours(3)).timestamp(),
            jti: Uuid::new_v4().to_string(),
            roles: vec!["user".to_string()],
        };

        assert!(jwt_manager.is_token_expired(&expired_claims));
    }

    #[test]
    fn test_jwt_invalid_token() {
        let jwt_manager = JwtManager::new("test_secret_key_123", 24);

        let result = jwt_manager.validate_token("invalid.token.here");
        assert!(result.is_err(), "Should reject invalid token");
    }

    #[test]
    fn test_jwt_wrong_secret() {
        let jwt_manager1 = JwtManager::new("secret1", 24);
        let jwt_manager2 = JwtManager::new("secret2", 24);

        let token = jwt_manager1
            .generate_token("user123", "test@example.com", vec!["user".to_string()])
            .unwrap();

        let result = jwt_manager2.validate_token(&token);
        assert!(
            result.is_err(),
            "Should reject token signed with different secret"
        );
    }

    #[test]
    fn test_jwt_role_checking() {
        let jwt_manager = JwtManager::new("test_secret_key_123", 24);

        let token = jwt_manager
            .generate_token(
                "user123",
                "test@example.com",
                vec!["user".to_string(), "admin".to_string()],
            )
            .unwrap();

        let claims = jwt_manager.validate_token(&token).unwrap();

        assert!(JwtManager::has_role(&claims, "user"));
        assert!(JwtManager::has_role(&claims, "admin"));
        assert!(!JwtManager::has_role(&claims, "superadmin"));

        assert!(JwtManager::has_any_role(&claims, &["admin", "superadmin"]));
        assert!(!JwtManager::has_any_role(&claims, &["superadmin", "owner"]));
    }

    #[test]
    fn test_jwt_user_id_extraction() {
        let jwt_manager = JwtManager::new("test_secret_key_123", 24);

        let token = jwt_manager
            .generate_token("user123", "test@example.com", vec!["user".to_string()])
            .unwrap();

        let claims = jwt_manager.validate_token(&token).unwrap();
        let user_id = JwtManager::get_user_id(&claims);

        assert_eq!(user_id, "user123");
    }

    // ===== User Storage Tests =====

    #[tokio::test]
    async fn test_user_registration() {
        let storage = Arc::new(MockUserStorage::new());

        let user = create_test_user("newuser@example.com", "password123", vec![
            "user".to_string(),
        ]);

        let user_id = storage.create_user(&user).await.unwrap();
        assert_eq!(user_id, user.id);

        let retrieved = storage.get_user(user_id).await.unwrap().unwrap();
        assert_eq!(retrieved.email, "newuser@example.com");
        assert_eq!(retrieved.roles, vec!["user"]);
    }

    #[tokio::test]
    async fn test_user_login_flow() {
        let storage = Arc::new(MockUserStorage::new());
        let jwt_manager = Arc::new(JwtManager::new("test_secret", 24));

        // Register user
        let user = create_test_user("login@example.com", "password123", vec![
            "user".to_string(),
        ]);
        storage.create_user(&user).await.unwrap();

        // Simulate login: get user by email
        let stored_user = storage
            .get_user_by_email("login@example.com")
            .await
            .unwrap()
            .unwrap();

        // Verify password
        assert!(password::verify_password("password123", &stored_user.password_hash).unwrap());

        // Generate token
        let token = jwt_manager
            .generate_token(
                &stored_user.id.to_string(),
                &stored_user.email,
                stored_user.roles.clone(),
            )
            .unwrap();

        // Validate token
        let claims = jwt_manager.validate_token(&token).unwrap();
        assert_eq!(claims.email, "login@example.com");
    }

    #[tokio::test]
    async fn test_duplicate_user_registration() {
        let storage = Arc::new(MockUserStorage::new());

        let user1 = create_test_user("duplicate@example.com", "password1", vec![
            "user".to_string(),
        ]);
        storage.create_user(&user1).await.unwrap();

        let user2 = create_test_user("duplicate@example.com", "password2", vec![
            "admin".to_string(),
        ]);
        let result = storage.create_user(&user2).await;

        assert!(result.is_err(), "Should reject duplicate email");
    }

    #[tokio::test]
    async fn test_user_update() {
        let storage = Arc::new(MockUserStorage::new());

        let mut user = create_test_user("update@example.com", "password123", vec![
            "user".to_string(),
        ]);
        let user_id = storage.create_user(&user).await.unwrap();

        // Update user
        user.name = "Updated Name".to_string();
        user.roles.push("admin".to_string());
        storage.update_user(user_id, &user).await.unwrap();

        let updated = storage.get_user(user_id).await.unwrap().unwrap();
        assert_eq!(updated.name, "Updated Name");
        assert_eq!(updated.roles.len(), 2);
    }

    #[tokio::test]
    async fn test_user_deletion() {
        let storage = Arc::new(MockUserStorage::new());

        let user = create_test_user("delete@example.com", "password123", vec![
            "user".to_string(),
        ]);
        let user_id = storage.create_user(&user).await.unwrap();

        storage.delete_user(user_id).await.unwrap();

        let retrieved = storage.get_user(user_id).await.unwrap();
        assert!(retrieved.is_none(), "User should be deleted");
    }

    #[tokio::test]
    async fn test_last_login_tracking() {
        let storage = Arc::new(MockUserStorage::new());

        let user = create_test_user("login@example.com", "password123", vec![
            "user".to_string(),
        ]);
        let user_id = storage.create_user(&user).await.unwrap();

        // Initially no last login
        let user_before = storage.get_user(user_id).await.unwrap().unwrap();
        assert!(user_before.last_login_at.is_none());

        // Update last login
        storage.update_last_login(user_id).await.unwrap();

        // Verify last login is set
        let user_after = storage.get_user(user_id).await.unwrap().unwrap();
        assert!(user_after.last_login_at.is_some());
    }

    #[tokio::test]
    async fn test_inactive_user() {
        let storage = Arc::new(MockUserStorage::new());

        let mut user = create_test_user("inactive@example.com", "password123", vec![
            "user".to_string(),
        ]);
        user.is_active = false;
        storage.create_user(&user).await.unwrap();

        let retrieved = storage
            .get_user_by_email("inactive@example.com")
            .await
            .unwrap()
            .unwrap();
        assert!(!retrieved.is_active, "User should be inactive");
    }

    // ===== Authorization Tests =====

    #[tokio::test]
    async fn test_permission_grant_and_check() {
        let auth_service = Arc::new(MockAuthorizationService::new());
        let user_id = Uuid::new_v4().to_string();

        auth_service.grant_permission(&user_id, "workflows:read");
        auth_service.grant_permission(&user_id, "workflows:write");
        auth_service.grant_permission(&user_id, "executions:read");

        assert!(
            auth_service
                .has_permission(&user_id, "workflows:read")
                .await
                .unwrap()
        );
        assert!(
            auth_service
                .has_permission(&user_id, "workflows:write")
                .await
                .unwrap()
        );
        assert!(
            auth_service
                .has_permission(&user_id, "executions:read")
                .await
                .unwrap()
        );
        assert!(
            !auth_service
                .has_permission(&user_id, "workflows:delete")
                .await
                .unwrap()
        );
    }

    #[tokio::test]
    async fn test_role_based_access_control() {
        let auth_service = Arc::new(MockAuthorizationService::new());
        let user_id = Uuid::new_v4().to_string();

        auth_service.grant_role(&user_id, "developer");
        auth_service.grant_role(&user_id, "viewer");

        assert!(auth_service.has_role(&user_id, "developer").await.unwrap());
        assert!(auth_service.has_role(&user_id, "viewer").await.unwrap());
        assert!(!auth_service.has_role(&user_id, "admin").await.unwrap());
    }

    #[tokio::test]
    async fn test_resource_level_permissions() {
        let auth_service = Arc::new(MockAuthorizationService::new());
        let user_id = Uuid::new_v4().to_string();
        let workflow_id = Uuid::new_v4();
        let execution_id = Uuid::new_v4();

        // Grant workflow permissions
        auth_service.grant_resource_access(&user_id, "workflow", workflow_id, "read");
        auth_service.grant_resource_access(&user_id, "workflow", workflow_id, "write");

        // Grant execution permissions
        auth_service.grant_resource_access(&user_id, "execution", execution_id, "read");

        // Check workflow access
        assert!(
            auth_service
                .can_access_resource(&user_id, "workflow", workflow_id, "read")
                .await
                .unwrap()
        );
        assert!(
            auth_service
                .can_access_resource(&user_id, "workflow", workflow_id, "write")
                .await
                .unwrap()
        );
        assert!(
            !auth_service
                .can_access_resource(&user_id, "workflow", workflow_id, "delete")
                .await
                .unwrap()
        );

        // Check execution access
        assert!(
            auth_service
                .can_access_resource(&user_id, "execution", execution_id, "read")
                .await
                .unwrap()
        );
        assert!(
            !auth_service
                .can_access_resource(&user_id, "execution", execution_id, "write")
                .await
                .unwrap()
        );
    }

    #[tokio::test]
    async fn test_permission_revocation() {
        let auth_service = Arc::new(MockAuthorizationService::new());
        let user_id = Uuid::new_v4().to_string();

        auth_service.grant_permission(&user_id, "workflows:write");
        assert!(
            auth_service
                .has_permission(&user_id, "workflows:write")
                .await
                .unwrap()
        );

        auth_service.revoke_permission(&user_id, "workflows:write");
        assert!(
            !auth_service
                .has_permission(&user_id, "workflows:write")
                .await
                .unwrap()
        );
    }

    #[tokio::test]
    async fn test_get_user_permissions() {
        let auth_service = Arc::new(MockAuthorizationService::new());
        let user_id = Uuid::new_v4().to_string();

        auth_service.grant_permission(&user_id, "workflows:read");
        auth_service.grant_permission(&user_id, "workflows:write");
        auth_service.grant_permission(&user_id, "executions:read");

        let permissions = auth_service
            .get_user_permissions(&user_id)
            .await
            .unwrap();

        assert_eq!(permissions.len(), 3);
        assert!(permissions.contains(&"workflows:read".to_string()));
        assert!(permissions.contains(&"workflows:write".to_string()));
        assert!(permissions.contains(&"executions:read".to_string()));
    }

    #[tokio::test]
    async fn test_multiple_users_isolation() {
        let auth_service = Arc::new(MockAuthorizationService::new());
        let user1_id = Uuid::new_v4().to_string();
        let user2_id = Uuid::new_v4().to_string();

        auth_service.grant_permission(&user1_id, "workflows:read");
        auth_service.grant_permission(&user2_id, "workflows:write");

        assert!(
            auth_service
                .has_permission(&user1_id, "workflows:read")
                .await
                .unwrap()
        );
        assert!(
            !auth_service
                .has_permission(&user1_id, "workflows:write")
                .await
                .unwrap()
        );

        assert!(
            !auth_service
                .has_permission(&user2_id, "workflows:read")
                .await
                .unwrap()
        );
        assert!(
            auth_service
                .has_permission(&user2_id, "workflows:write")
                .await
                .unwrap()
        );
    }

    #[tokio::test]
    async fn test_hierarchical_permissions() {
        let auth_service = Arc::new(MockAuthorizationService::new());
        let admin_id = Uuid::new_v4().to_string();
        let user_id = Uuid::new_v4().to_string();

        // Admin has all permissions
        auth_service.grant_role(&admin_id, "admin");
        auth_service.grant_permission(&admin_id, "workflows:read");
        auth_service.grant_permission(&admin_id, "workflows:write");
        auth_service.grant_permission(&admin_id, "workflows:delete");
        auth_service.grant_permission(&admin_id, "executions:read");
        auth_service.grant_permission(&admin_id, "executions:write");
        auth_service.grant_permission(&admin_id, "executions:delete");

        // Regular user has limited permissions
        auth_service.grant_role(&user_id, "user");
        auth_service.grant_permission(&user_id, "workflows:read");
        auth_service.grant_permission(&user_id, "executions:read");

        // Verify admin permissions
        assert!(auth_service.has_role(&admin_id, "admin").await.unwrap());
        assert!(
            auth_service
                .has_permission(&admin_id, "workflows:delete")
                .await
                .unwrap()
        );

        // Verify user permissions
        assert!(auth_service.has_role(&user_id, "user").await.unwrap());
        assert!(
            !auth_service
                .has_permission(&user_id, "workflows:delete")
                .await
                .unwrap()
        );
    }

    // ===== Integration Tests =====

    #[tokio::test]
    async fn test_complete_auth_flow() {
        let user_storage = Arc::new(MockUserStorage::new());
        let auth_service = Arc::new(MockAuthorizationService::new());
        let jwt_manager = Arc::new(JwtManager::new("test_secret", 24));

        // 1. Register user
        let user = create_test_user("complete@example.com", "password123", vec![
            "user".to_string(),
        ]);
        let user_id = user_storage.create_user(&user).await.unwrap();

        // 2. Setup permissions
        let user_id_str = user_id.to_string();
        auth_service.grant_role(&user_id_str, "user");
        auth_service.grant_permission(&user_id_str, "workflows:read");

        // 3. Login simulation
        let stored_user = user_storage
            .get_user_by_email("complete@example.com")
            .await
            .unwrap()
            .unwrap();

        assert!(password::verify_password("password123", &stored_user.password_hash).unwrap());

        // 4. Generate token
        let token = jwt_manager
            .generate_token(
                &user_id_str,
                &stored_user.email,
                stored_user.roles.clone(),
            )
            .unwrap();

        // 5. Validate token
        let claims = jwt_manager.validate_token(&token).unwrap();
        assert_eq!(claims.sub, user_id_str);

        // 6. Check authorization
        assert!(
            auth_service
                .has_permission(&user_id_str, "workflows:read")
                .await
                .unwrap()
        );
        assert!(auth_service.has_role(&user_id_str, "user").await.unwrap());

        // 7. Update last login
        user_storage.update_last_login(user_id).await.unwrap();
    }

    #[tokio::test]
    async fn test_failed_login_invalid_password() {
        let storage = Arc::new(MockUserStorage::new());

        let user = create_test_user("test@example.com", "correctpassword", vec![
            "user".to_string(),
        ]);
        storage.create_user(&user).await.unwrap();

        let stored_user = storage
            .get_user_by_email("test@example.com")
            .await
            .unwrap()
            .unwrap();

        // Wrong password should fail
        assert!(!password::verify_password("wrongpassword", &stored_user.password_hash).unwrap());
    }

    #[tokio::test]
    async fn test_failed_login_nonexistent_user() {
        let storage = Arc::new(MockUserStorage::new());

        let result = storage
            .get_user_by_email("nonexistent@example.com")
            .await
            .unwrap();

        assert!(result.is_none(), "Should return None for nonexistent user");
    }

    // ===== Error Handling Tests =====

    #[tokio::test]
    async fn test_storage_failure_handling() {
        let storage = Arc::new(MockUserStorage::new());

        let user = create_test_user("test@example.com", "password123", vec![
            "user".to_string(),
        ]);
        storage.create_user(&user).await.unwrap();

        storage.fail_operations();

        let result = storage.get_user(user.id).await;
        assert!(result.is_err(), "Should fail when failure mode enabled");
    }

    #[tokio::test]
    async fn test_authorization_failure_handling() {
        let auth_service = Arc::new(MockAuthorizationService::new());
        let user_id = Uuid::new_v4().to_string();

        auth_service.grant_permission(&user_id, "workflows:read");

        auth_service.fail_operations();

        let result = auth_service.has_permission(&user_id, "workflows:read").await;
        assert!(
            result.is_err(),
            "Should fail when failure mode enabled"
        );
    }
}
