pub mod api_key;
pub mod session;

use axum::extract::FromRequestParts;
use axum::http::request::Parts;
use serde::{Deserialize, Serialize};

use crate::app::AppState;
use crate::error::AppError;

/// The authenticated user, resolved from either session cookie or API key.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuthUser {
    pub id: String,
    pub email: String,
    pub name: String,
    pub role: String,
    pub auth_method: String,
    /// Set when auth came from an API key
    pub project_id: Option<String>,
    /// Set when auth came from an API key
    pub api_key_type: Option<String>,
}

/// Axum extractor that resolves the current user from session cookie or API key.
impl FromRequestParts<AppState> for AuthUser {
    type Rejection = AppError;

    async fn from_request_parts(
        parts: &mut Parts,
        state: &AppState,
    ) -> Result<Self, Self::Rejection> {
        // Try API key auth first (Authorization: Bearer flk_...)
        if let Some(auth_header) = parts.headers.get("authorization") {
            if let Ok(header_str) = auth_header.to_str() {
                if let Some(raw_key) = header_str.strip_prefix("Bearer ") {
                    return api_key::resolve_auth(state, raw_key).await;
                }
            }
        }

        // Try session cookie auth
        if let Some(cookie_header) = parts.headers.get("cookie") {
            if let Ok(cookie_str) = cookie_header.to_str() {
                if let Some(user) = session::resolve_session(state, cookie_str).await? {
                    return Ok(user);
                }
            }
        }

        Err(AppError::unauthorized("Unauthorized"))
    }
}

/// SDK-only auth: requires a valid SDK or management API key.
#[derive(Debug, Clone)]
pub struct SdkAuth(pub AuthUser);

impl FromRequestParts<AppState> for SdkAuth {
    type Rejection = AppError;

    async fn from_request_parts(
        parts: &mut Parts,
        state: &AppState,
    ) -> Result<Self, Self::Rejection> {
        let user = AuthUser::from_request_parts(parts, state).await?;
        match user.api_key_type.as_deref() {
            Some("sdk") | Some("management") => Ok(SdkAuth(user)),
            _ => Err(AppError::unauthorized("SDK or management API key required")),
        }
    }
}
