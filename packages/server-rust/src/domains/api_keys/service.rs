use base64::Engine;
use fred::prelude::*;
use rand::RngCore;
use serde::Serialize;
use sha2::{Digest, Sha256};
use sqlx::PgPool;
use uuid::Uuid;

use crate::auth::AuthUser;
use crate::error::AppError;

use super::repo::{self, ApiKeyRow};

#[derive(Debug, Serialize)]
pub struct CreatedApiKey {
    #[serde(flatten)]
    pub key: ApiKeyRow,
    pub raw_key: String,
}

pub async fn list_api_keys(db: &PgPool, project_id: Uuid) -> Result<Vec<ApiKeyRow>, AppError> {
    repo::find_by_project(db, project_id)
        .await
        .map_err(AppError::from)
}

pub async fn create_api_key(
    db: &PgPool,
    project_id: Uuid,
    name: &str,
    key_type: &str,
    environment_id: Option<Uuid>,
    actor: &AuthUser,
) -> Result<CreatedApiKey, AppError> {
    // Generate raw key
    let prefix = if key_type == "sdk" {
        "flk_sdk_"
    } else {
        "flk_mgmt_"
    };
    let mut random_bytes = [0u8; 24];
    rand::thread_rng().fill_bytes(&mut random_bytes);
    let random = base64::engine::general_purpose::URL_SAFE_NO_PAD.encode(&random_bytes);
    let raw = format!("{}{}", prefix, random);

    // Hash the key
    let hash = format!("{:x}", Sha256::digest(raw.as_bytes()));

    // Display prefix (first 12 chars)
    let key_prefix_display = &raw[..12];

    // Parse the actor id (String) to Uuid
    let actor_id: Uuid = actor.id.parse().map_err(|_| AppError::validation("Invalid actor ID"))?;

    let key = repo::create(
        db,
        project_id,
        name,
        key_prefix_display,
        &hash,
        key_type,
        environment_id,
        actor_id,
    )
    .await?;

    Ok(CreatedApiKey {
        key,
        raw_key: raw,
    })
}

pub async fn revoke_api_key(
    db: &PgPool,
    redis: &fred::clients::RedisPool,
    id: Uuid,
) -> Result<(), AppError> {
    let deleted = repo::remove(db, id).await?;

    match deleted {
        Some(key_row) => {
            // Invalidate cache using the key hash
            let cache_key = format!("flick:apikey:{}", key_row.key_hash);
            let _: Result<(), _> = redis.del::<(), _>(&cache_key).await;
            Ok(())
        }
        None => Err(AppError::not_found("API key", Some(&id.to_string()))),
    }
}
