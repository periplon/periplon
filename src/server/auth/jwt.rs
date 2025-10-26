// JWT token management

#[cfg(feature = "server")]
use chrono::{Duration, Utc};
#[cfg(feature = "server")]
use jsonwebtoken::{decode, encode, Algorithm, DecodingKey, EncodingKey, Header, Validation};
#[cfg(feature = "server")]
use serde::{Deserialize, Serialize};
#[cfg(feature = "server")]
use uuid::Uuid;

#[cfg(feature = "server")]
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Claims {
    pub sub: String,        // Subject (user ID)
    pub email: String,      // User email
    pub exp: i64,           // Expiration time
    pub iat: i64,           // Issued at
    pub jti: String,        // JWT ID
    pub roles: Vec<String>, // User roles
}

#[cfg(feature = "server")]
pub struct JwtManager {
    encoding_key: EncodingKey,
    decoding_key: DecodingKey,
    algorithm: Algorithm,
    expiration_hours: i64,
}

#[cfg(feature = "server")]
impl JwtManager {
    /// Create a new JWT manager with a secret key
    pub fn new(secret: &str, expiration_hours: i64) -> Self {
        Self {
            encoding_key: EncodingKey::from_secret(secret.as_bytes()),
            decoding_key: DecodingKey::from_secret(secret.as_bytes()),
            algorithm: Algorithm::HS256,
            expiration_hours,
        }
    }

    /// Generate a JWT token for a user
    pub fn generate_token(
        &self,
        user_id: &str,
        email: &str,
        roles: Vec<String>,
    ) -> Result<String, jsonwebtoken::errors::Error> {
        let now = Utc::now();
        let expiration = now + Duration::hours(self.expiration_hours);

        let claims = Claims {
            sub: user_id.to_string(),
            email: email.to_string(),
            exp: expiration.timestamp(),
            iat: now.timestamp(),
            jti: Uuid::new_v4().to_string(),
            roles,
        };

        encode(&Header::new(self.algorithm), &claims, &self.encoding_key)
    }

    /// Validate and decode a JWT token
    pub fn validate_token(&self, token: &str) -> Result<Claims, jsonwebtoken::errors::Error> {
        let mut validation = Validation::new(self.algorithm);
        validation.validate_exp = true;

        let token_data = decode::<Claims>(token, &self.decoding_key, &validation)?;
        Ok(token_data.claims)
    }

    /// Check if a token is expired
    pub fn is_token_expired(&self, claims: &Claims) -> bool {
        let now = Utc::now().timestamp();
        claims.exp < now
    }

    /// Extract user ID from claims
    pub fn get_user_id(claims: &Claims) -> &str {
        &claims.sub
    }

    /// Check if user has a specific role
    pub fn has_role(claims: &Claims, role: &str) -> bool {
        claims.roles.iter().any(|r| r == role)
    }

    /// Check if user has any of the specified roles
    pub fn has_any_role(claims: &Claims, roles: &[&str]) -> bool {
        claims.roles.iter().any(|r| roles.contains(&r.as_str()))
    }
}

#[cfg(all(test, feature = "server"))]
mod tests {
    use super::*;

    #[test]
    fn test_jwt_generation_and_validation() {
        let manager = JwtManager::new("test_secret_key_12345", 24);

        // Generate token
        let token = manager
            .generate_token(
                "user123",
                "user@example.com",
                vec!["user".to_string(), "admin".to_string()],
            )
            .unwrap();

        assert!(!token.is_empty());

        // Validate token
        let claims = manager.validate_token(&token).unwrap();
        assert_eq!(claims.sub, "user123");
        assert_eq!(claims.email, "user@example.com");
        assert!(claims.roles.contains(&"user".to_string()));
        assert!(claims.roles.contains(&"admin".to_string()));

        // Check roles
        assert!(JwtManager::has_role(&claims, "admin"));
        assert!(JwtManager::has_any_role(&claims, &["admin", "superuser"]));
        assert!(!JwtManager::has_role(&claims, "superuser"));
    }

    #[test]
    fn test_expired_token() {
        let manager = JwtManager::new("test_secret_key_12345", -1); // Expired 1 hour ago

        let token = manager
            .generate_token("user123", "user@example.com", vec!["user".to_string()])
            .unwrap();

        // This should fail because token is expired
        let result = manager.validate_token(&token);
        assert!(result.is_err());
    }

    #[test]
    fn test_invalid_signature() {
        let manager1 = JwtManager::new("secret1", 24);
        let manager2 = JwtManager::new("secret2", 24);

        let token = manager1
            .generate_token("user123", "user@example.com", vec!["user".to_string()])
            .unwrap();

        // This should fail because signature doesn't match
        let result = manager2.validate_token(&token);
        assert!(result.is_err());
    }
}
