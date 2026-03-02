use axum::extract::Request;
use axum::http::HeaderValue;
use axum::middleware::Next;
use axum::response::Response;
use fred::prelude::*;

use crate::app::AppState;
use crate::cache::keys;
use crate::error::AppError;

pub struct RateLimitConfig {
    pub prefix: &'static str,
    pub max: i64,
    pub window_sec: i64,
    pub message: &'static str,
}

pub const SDK_EVALUATE_LIMIT: RateLimitConfig = RateLimitConfig {
    prefix: "eval",
    max: 1000,
    window_sec: 10,
    message: "Too many evaluation requests. Use the SDK — it polls and caches flags locally.",
};

pub const LOGIN_IP_LIMIT: RateLimitConfig = RateLimitConfig {
    prefix: "login",
    max: 5,
    window_sec: 60,
    message: "Too many login attempts.",
};

pub const LOGIN_EMAIL_LIMIT: RateLimitConfig = RateLimitConfig {
    prefix: "login-email",
    max: 10,
    window_sec: 300,
    message: "Too many login attempts for this email.",
};

/// Check rate limit using Redis INCR + EXPIRE sliding window.
pub async fn check_rate_limit(
    redis: &fred::clients::RedisPool,
    config: &RateLimitConfig,
    identifier: &str,
) -> Result<RateLimitHeaders, AppError> {
    let key = keys::rate_limit_key(config.prefix, identifier);

    let count: i64 = redis.incr(&key).await.map_err(|e| {
        tracing::warn!(error = %e, "Rate limit INCR failed");
        AppError::Internal(anyhow::anyhow!("Rate limit check failed"))
    })?;

    if count == 1 {
        let _ = redis.expire::<(), _>(&key, config.window_sec).await;
    }

    let ttl: i64 = redis.ttl(&key).await.unwrap_or(config.window_sec);
    let remaining = (config.max - count).max(0);
    let reset = chrono::Utc::now().timestamp() + ttl.max(0);

    if count > config.max {
        let retry_after = ttl.max(1);
        return Err(AppError::rate_limited(
            format!("{} Retry after {}s.", config.message, retry_after),
            retry_after,
        ));
    }

    Ok(RateLimitHeaders {
        limit: config.max,
        remaining,
        reset,
    })
}

pub struct RateLimitHeaders {
    pub limit: i64,
    pub remaining: i64,
    pub reset: i64,
}

impl RateLimitHeaders {
    pub fn apply(&self, response: &mut Response) {
        let headers = response.headers_mut();
        headers.insert(
            "X-RateLimit-Limit",
            HeaderValue::from_str(&self.limit.to_string()).unwrap(),
        );
        headers.insert(
            "X-RateLimit-Remaining",
            HeaderValue::from_str(&self.remaining.to_string()).unwrap(),
        );
        headers.insert(
            "X-RateLimit-Reset",
            HeaderValue::from_str(&self.reset.to_string()).unwrap(),
        );
    }
}

/// Middleware for SDK evaluate rate limiting (keyed by last 8 chars of bearer token).
pub async fn evaluate_rate_limit(
    axum::extract::State(state): axum::extract::State<AppState>,
    request: Request,
    next: Next,
) -> Result<Response, AppError> {
    let identifier = request
        .headers()
        .get("authorization")
        .and_then(|v| v.to_str().ok())
        .filter(|v| v.starts_with("Bearer "))
        .map(|v| &v[v.len().saturating_sub(8)..]);

    if let Some(id) = identifier {
        let headers = check_rate_limit(&state.redis, &SDK_EVALUATE_LIMIT, id).await?;
        let mut response = next.run(request).await;
        headers.apply(&mut response);
        Ok(response)
    } else {
        Ok(next.run(request).await)
    }
}
