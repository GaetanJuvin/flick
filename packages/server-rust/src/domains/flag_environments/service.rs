use serde_json::Value;
use sqlx::PgPool;
use uuid::Uuid;

use crate::cache;
use crate::error::AppError;

use super::repo::{self, FlagEnvironmentRow};

pub async fn get_flag_environments(
    db: &PgPool,
    flag_id: Uuid,
) -> Result<Vec<FlagEnvironmentRow>, AppError> {
    repo::find_by_flag(db, flag_id)
        .await
        .map_err(AppError::from)
}

pub async fn get_flag_environment(
    db: &PgPool,
    flag_id: Uuid,
    env_id: Uuid,
) -> Result<FlagEnvironmentRow, AppError> {
    repo::find_by_flag_and_env(db, flag_id, env_id)
        .await?
        .ok_or_else(|| {
            AppError::not_found(
                "FlagEnvironment",
                Some(&format!("flag={},env={}", flag_id, env_id)),
            )
        })
}

pub struct UpdateFlagEnvInput {
    pub enabled: Option<bool>,
    pub gate_config: Option<Value>,
}

pub async fn update_flag_environment(
    db: &PgPool,
    redis: &fred::clients::RedisPool,
    flag_id: Uuid,
    env_id: Uuid,
    input: UpdateFlagEnvInput,
) -> Result<FlagEnvironmentRow, AppError> {
    let fe = repo::find_by_flag_and_env(db, flag_id, env_id)
        .await?
        .ok_or_else(|| {
            AppError::not_found(
                "FlagEnvironment",
                Some(&format!("flag={},env={}", flag_id, env_id)),
            )
        })?;

    let updated = repo::update(db, fe.id, input.enabled, input.gate_config)
        .await?
        .ok_or_else(|| {
            AppError::not_found("FlagEnvironment", Some(&fe.id.to_string()))
        })?;

    cache::invalidate_env_flags(redis, &env_id.to_string()).await;

    Ok(updated)
}

pub async fn toggle_flag_environment(
    db: &PgPool,
    redis: &fred::clients::RedisPool,
    flag_id: Uuid,
    env_id: Uuid,
) -> Result<FlagEnvironmentRow, AppError> {
    let fe = repo::find_by_flag_and_env(db, flag_id, env_id)
        .await?
        .ok_or_else(|| {
            AppError::not_found(
                "FlagEnvironment",
                Some(&format!("flag={},env={}", flag_id, env_id)),
            )
        })?;

    let toggled = repo::toggle(db, fe.id)
        .await?
        .ok_or_else(|| {
            AppError::not_found("FlagEnvironment", Some(&fe.id.to_string()))
        })?;

    cache::invalidate_env_flags(redis, &env_id.to_string()).await;

    Ok(toggled)
}
