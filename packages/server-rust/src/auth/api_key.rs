use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};

use crate::app::AppState;
use crate::auth::AuthUser;
use crate::cache::{self, keys, CACHE_TTL_API_KEY};
use crate::error::AppError;

#[derive(Debug, Clone, Serialize, Deserialize)]
struct CachedApiKey {
    id: String,
    project_id: String,
    key_type: String,
    user: CachedUser,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct CachedUser {
    id: String,
    email: String,
    name: String,
    role: String,
    auth_method: String,
}

fn hash_api_key(raw: &str) -> String {
    let mut hasher = Sha256::new();
    hasher.update(raw.as_bytes());
    format!("{:x}", hasher.finalize())
}

pub async fn resolve_auth(state: &AppState, raw_key: &str) -> Result<AuthUser, AppError> {
    let key_hash = hash_api_key(raw_key);
    let cache_key = keys::api_key_hash_key(&key_hash);

    // Try Redis cache first
    if let Some(cached) = cache::cache_get::<CachedApiKey>(&state.redis, &cache_key).await {
        // Fire-and-forget last_used_at update
        let db = state.db.clone();
        let id = cached.id.clone();
        tokio::spawn(async move {
            let _ = sqlx::query("UPDATE api_keys SET last_used_at = now() WHERE id = $1")
                .bind(&id)
                .execute(&db)
                .await;
        });

        return Ok(AuthUser {
            id: cached.user.id,
            email: cached.user.email,
            name: cached.user.name,
            role: cached.user.role,
            auth_method: cached.user.auth_method,
            project_id: Some(cached.project_id),
            api_key_type: Some(cached.key_type),
        });
    }

    // Cache miss — hit Postgres
    let api_key = sqlx::query_as::<_, ApiKeyRow>(
        "SELECT ak.id, ak.project_id, ak.type as key_type, ak.created_by
         FROM api_keys ak WHERE ak.key_hash = $1"
    )
    .bind(&key_hash)
    .fetch_optional(&state.db)
    .await?;

    let api_key = match api_key {
        Some(k) => k,
        None => return Err(AppError::unauthorized("Invalid API key")),
    };

    let user = sqlx::query_as::<_, UserRow>(
        "SELECT id, email, name, role, auth_method FROM users WHERE id = $1"
    )
    .bind(&api_key.created_by)
    .fetch_optional(&state.db)
    .await?;

    let user = match user {
        Some(u) => u,
        None => return Err(AppError::unauthorized("Invalid API key")),
    };

    // Cache the resolved key
    let entry = CachedApiKey {
        id: api_key.id.to_string(),
        project_id: api_key.project_id.to_string(),
        key_type: api_key.key_type.clone(),
        user: CachedUser {
            id: user.id.to_string(),
            email: user.email.clone(),
            name: user.name.clone(),
            role: user.role.clone(),
            auth_method: user.auth_method.clone(),
        },
    };
    cache::cache_set(&state.redis, &cache_key, &entry, CACHE_TTL_API_KEY).await;

    // Fire-and-forget last_used_at update
    let db = state.db.clone();
    let ak_id = api_key.id.to_string();
    tokio::spawn(async move {
        let _ = sqlx::query("UPDATE api_keys SET last_used_at = now() WHERE id = $1")
            .bind(&ak_id)
            .execute(&db)
            .await;
    });

    Ok(AuthUser {
        id: user.id.to_string(),
        email: user.email,
        name: user.name,
        role: user.role,
        auth_method: user.auth_method,
        project_id: Some(api_key.project_id.to_string()),
        api_key_type: Some(api_key.key_type),
    })
}

#[derive(sqlx::FromRow)]
struct ApiKeyRow {
    id: uuid::Uuid,
    project_id: uuid::Uuid,
    key_type: String,
    created_by: uuid::Uuid,
}

#[derive(sqlx::FromRow)]
struct UserRow {
    id: uuid::Uuid,
    email: String,
    name: String,
    role: String,
    auth_method: String,
}
