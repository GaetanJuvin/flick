use base64::Engine;
use serde::{Deserialize, Serialize};

use crate::app::AppState;
use crate::auth::AuthUser;
use crate::error::AppError;

#[derive(Debug, Serialize, Deserialize)]
pub struct SessionData {
    pub id: String,
    pub email: String,
    pub name: String,
    pub role: String,
    pub auth_method: String,
}

/// Parse session cookie and resolve the user from the database.
pub async fn resolve_session(
    state: &AppState,
    cookie_header: &str,
) -> Result<Option<AuthUser>, AppError> {
    let session_value = cookie_header
        .split(';')
        .filter_map(|pair| {
            let mut parts = pair.trim().splitn(2, '=');
            let name = parts.next()?.trim();
            let value = parts.next()?.trim();
            if name == "session" { Some(value.to_string()) } else { None }
        })
        .next();

    let session_value = match session_value {
        Some(v) => v,
        None => return Ok(None),
    };

    let decoded = match base64::engine::general_purpose::STANDARD.decode(&session_value) {
        Ok(d) => d,
        Err(_) => return Ok(None),
    };

    let parsed: SessionData = match serde_json::from_slice(&decoded) {
        Ok(p) => p,
        Err(_) => return Ok(None),
    };

    // Verify user still exists in DB
    let user = sqlx::query_as::<_, UserRow>(
        "SELECT id, email, name, role, auth_method FROM users WHERE id = $1"
    )
    .bind(&parsed.id)
    .fetch_optional(&state.db)
    .await?;

    match user {
        Some(row) => Ok(Some(AuthUser {
            id: row.id.to_string(),
            email: row.email,
            name: row.name,
            role: row.role,
            auth_method: row.auth_method,
            project_id: None,
            api_key_type: None,
        })),
        None => Ok(None),
    }
}

/// Create a session cookie value from a user.
pub fn create_session_cookie(user: &AuthUser) -> String {
    let data = SessionData {
        id: user.id.clone(),
        email: user.email.clone(),
        name: user.name.clone(),
        role: user.role.clone(),
        auth_method: user.auth_method.clone(),
    };
    let json = serde_json::to_string(&data).unwrap();
    base64::engine::general_purpose::STANDARD.encode(json.as_bytes())
}

#[derive(sqlx::FromRow)]
struct UserRow {
    id: uuid::Uuid,
    email: String,
    name: String,
    role: String,
    auth_method: String,
}
