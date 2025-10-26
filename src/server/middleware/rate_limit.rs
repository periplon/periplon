// Rate limiting middleware

#[cfg(feature = "server")]
use axum::{
    body::Body,
    extract::Request,
    http::{HeaderMap, HeaderValue, StatusCode},
    middleware::Next,
    response::{IntoResponse, Response},
    Json,
};
#[cfg(feature = "server")]
use serde_json::json;
#[cfg(feature = "server")]
use std::collections::HashMap;
#[cfg(feature = "server")]
use std::sync::{Arc, Mutex};
#[cfg(feature = "server")]
use std::time::{Duration, Instant};

#[cfg(feature = "server")]
use crate::server::config::RateLimitConfig;

/// Simple in-memory rate limiter
#[cfg(feature = "server")]
#[derive(Clone)]
pub struct RateLimiter {
    config: RateLimitConfig,
    // Track requests per key (IP, user, API key)
    // key -> (count, window_start)
    requests: Arc<Mutex<HashMap<String, (u32, Instant)>>>,
    window_duration: Duration,
}

#[cfg(feature = "server")]
impl RateLimiter {
    pub fn new(config: RateLimitConfig) -> Self {
        Self {
            config,
            requests: Arc::new(Mutex::new(HashMap::new())),
            window_duration: Duration::from_secs(60), // 1 minute window
        }
    }

    /// Check if request should be rate limited
    /// Returns (allowed, remaining, reset_seconds)
    fn check_rate_limit(&self, key: &str, limit: u32) -> (bool, u32, u64) {
        let mut requests = self.requests.lock().unwrap();
        let now = Instant::now();

        // Get or create entry for this key
        let entry = requests.entry(key.to_string()).or_insert((0, now));

        // Check if window has expired
        if now.duration_since(entry.1) > self.window_duration {
            // Reset window
            *entry = (1, now);
            return (true, limit - 1, self.window_duration.as_secs());
        }

        // Increment counter
        entry.0 += 1;

        let remaining = limit.saturating_sub(entry.0);

        let reset_seconds = self.window_duration.as_secs() - now.duration_since(entry.1).as_secs();

        (entry.0 <= limit, remaining, reset_seconds)
    }

    /// Get IP address from request
    fn get_client_ip(headers: &HeaderMap) -> Option<String> {
        // Try X-Forwarded-For first (for proxies)
        if let Some(forwarded) = headers.get("x-forwarded-for") {
            if let Ok(forwarded_str) = forwarded.to_str() {
                // Take first IP in the chain
                return forwarded_str
                    .split(',')
                    .next()
                    .map(|s| s.trim().to_string());
            }
        }

        // Try X-Real-IP
        if let Some(real_ip) = headers.get("x-real-ip") {
            if let Ok(ip_str) = real_ip.to_str() {
                return Some(ip_str.to_string());
            }
        }

        None
    }

    /// Get API key from request
    fn get_api_key(headers: &HeaderMap) -> Option<String> {
        headers
            .get("x-api-key")
            .and_then(|v| v.to_str().ok())
            .map(|s| s.to_string())
    }

    /// Get user ID from JWT claims in request extensions
    fn get_user_id(request: &Request<Body>) -> Option<String> {
        use crate::server::auth::jwt::Claims;

        request
            .extensions()
            .get::<Claims>()
            .map(|claims| claims.sub.clone())
    }
}

/// Rate limiting middleware
#[cfg(feature = "server")]
pub async fn rate_limit_middleware(
    limiter: RateLimiter,
    request: Request<Body>,
    next: Next,
) -> Result<Response, Response> {
    if !limiter.config.enabled {
        return Ok(next.run(request).await);
    }

    let headers = request.headers();

    // Track the most restrictive limit for response headers
    let mut limit: Option<u32> = None;
    let mut remaining: Option<u32> = None;
    let mut reset: Option<u64> = None;

    // Check global rate limit (per IP)
    if let Some(ip) = RateLimiter::get_client_ip(headers) {
        let (allowed, rem, rst) = limiter.check_rate_limit(
            &format!("global:{}", ip),
            limiter.config.per_ip_requests_per_minute,
        );

        limit = Some(limiter.config.per_ip_requests_per_minute);
        remaining = Some(rem);
        reset = Some(rst);

        if !allowed {
            return Err((
                StatusCode::TOO_MANY_REQUESTS,
                [
                    (
                        "X-RateLimit-Limit",
                        limiter.config.per_ip_requests_per_minute.to_string(),
                    ),
                    ("X-RateLimit-Remaining", "0".to_string()),
                    ("X-RateLimit-Reset", rst.to_string()),
                    ("Retry-After", rst.to_string()),
                ],
                Json(json!({
                    "error": "Rate limit exceeded",
                    "message": "Too many requests from your IP",
                    "retry_after": rst,
                })),
            )
                .into_response());
        }
    }

    // Check API key rate limit
    if let Some(api_key) = RateLimiter::get_api_key(headers) {
        let (allowed, rem, rst) = limiter.check_rate_limit(
            &format!("api_key:{}", api_key),
            limiter.config.per_api_key_requests_per_minute,
        );

        // Use the most restrictive limit
        if let Some(current_remaining) = remaining {
            if rem < current_remaining {
                limit = Some(limiter.config.per_api_key_requests_per_minute);
                remaining = Some(rem);
                reset = Some(rst);
            }
        } else {
            limit = Some(limiter.config.per_api_key_requests_per_minute);
            remaining = Some(rem);
            reset = Some(rst);
        }

        if !allowed {
            return Err((
                StatusCode::TOO_MANY_REQUESTS,
                [
                    (
                        "X-RateLimit-Limit",
                        limiter.config.per_api_key_requests_per_minute.to_string(),
                    ),
                    ("X-RateLimit-Remaining", "0".to_string()),
                    ("X-RateLimit-Reset", rst.to_string()),
                    ("Retry-After", rst.to_string()),
                ],
                Json(json!({
                    "error": "Rate limit exceeded",
                    "message": "Too many requests for this API key",
                    "retry_after": rst,
                })),
            )
                .into_response());
        }
    }

    // Check user rate limit (if authenticated)
    if let Some(user_id) = RateLimiter::get_user_id(&request) {
        let (allowed, rem, rst) = limiter.check_rate_limit(
            &format!("user:{}", user_id),
            limiter.config.per_user_requests_per_minute,
        );

        // Use the most restrictive limit
        if let Some(current_remaining) = remaining {
            if rem < current_remaining {
                limit = Some(limiter.config.per_user_requests_per_minute);
                remaining = Some(rem);
                reset = Some(rst);
            }
        } else {
            limit = Some(limiter.config.per_user_requests_per_minute);
            remaining = Some(rem);
            reset = Some(rst);
        }

        if !allowed {
            return Err((
                StatusCode::TOO_MANY_REQUESTS,
                [
                    (
                        "X-RateLimit-Limit",
                        limiter.config.per_user_requests_per_minute.to_string(),
                    ),
                    ("X-RateLimit-Remaining", "0".to_string()),
                    ("X-RateLimit-Reset", rst.to_string()),
                    ("Retry-After", rst.to_string()),
                ],
                Json(json!({
                    "error": "Rate limit exceeded",
                    "message": "Too many requests for your account",
                    "retry_after": rst,
                })),
            )
                .into_response());
        }
    }

    // Request allowed, add rate limit headers to response
    let mut response = next.run(request).await;

    // Add X-RateLimit headers if we have rate limit info
    if let (Some(lim), Some(rem), Some(rst)) = (limit, remaining, reset) {
        let headers = response.headers_mut();

        if let Ok(value) = HeaderValue::from_str(&lim.to_string()) {
            headers.insert("X-RateLimit-Limit", value);
        }

        if let Ok(value) = HeaderValue::from_str(&rem.to_string()) {
            headers.insert("X-RateLimit-Remaining", value);
        }

        if let Ok(value) = HeaderValue::from_str(&rst.to_string()) {
            headers.insert("X-RateLimit-Reset", value);
        }
    }

    Ok(response)
}

#[cfg(all(test, feature = "server"))]
mod tests {
    use super::*;

    #[test]
    fn test_rate_limiter() {
        let config = RateLimitConfig {
            enabled: true,
            global_requests_per_minute: 100,
            per_user_requests_per_minute: 50,
            per_api_key_requests_per_minute: 200,
            per_ip_requests_per_minute: 60,
        };

        let limiter = RateLimiter::new(config);

        // First request should be allowed
        let (allowed, remaining, _) = limiter.check_rate_limit("test_key", 10);
        assert!(allowed);
        assert_eq!(remaining, 9);

        // Subsequent requests should decrement remaining
        for i in 0..9 {
            let (allowed, remaining, _) = limiter.check_rate_limit("test_key", 10);
            assert!(allowed);
            assert_eq!(remaining, 9 - i - 1);
        }

        // 11th request should be denied
        let (allowed, remaining, _) = limiter.check_rate_limit("test_key", 10);
        assert!(!allowed);
        assert_eq!(remaining, 0);
    }
}
