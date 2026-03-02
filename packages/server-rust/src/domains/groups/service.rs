use fred::prelude::*;
use serde_json::Value;
use sqlx::PgPool;
use uuid::Uuid;

use crate::auth::AuthUser;
use crate::error::AppError;

use super::repo::{self, FlagGroupRow, GroupRow};

pub async fn list_groups(db: &PgPool, project_id: Uuid) -> Result<Vec<GroupRow>, AppError> {
    repo::find_by_project(db, project_id)
        .await
        .map_err(AppError::from)
}

pub async fn get_group(db: &PgPool, id: Uuid) -> Result<GroupRow, AppError> {
    repo::find_by_id(db, id)
        .await?
        .ok_or_else(|| AppError::not_found("Group", Some(&id.to_string())))
}

pub async fn create_group(
    db: &PgPool,
    project_id: Uuid,
    name: &str,
    slug: &str,
    description: &str,
    rules: Value,
) -> Result<GroupRow, AppError> {
    // Check for slug uniqueness
    if let Some(_existing) = repo::find_by_slug(db, project_id, slug).await? {
        return Err(AppError::Conflict(format!(
            "Group with slug '{}' already exists",
            slug
        )));
    }

    repo::create(db, project_id, name, slug, description, rules)
        .await
        .map_err(AppError::from)
}

pub async fn update_group(
    db: &PgPool,
    id: Uuid,
    name: Option<&str>,
    description: Option<&str>,
    rules: Option<Value>,
) -> Result<GroupRow, AppError> {
    repo::update(db, id, name, description, rules)
        .await?
        .ok_or_else(|| AppError::not_found("Group", Some(&id.to_string())))
}

pub async fn delete_group(db: &PgPool, id: Uuid) -> Result<bool, AppError> {
    let deleted = repo::remove(db, id).await?;
    if !deleted {
        return Err(AppError::not_found("Group", Some(&id.to_string())));
    }
    Ok(deleted)
}

pub async fn get_groups_for_flag_env(
    db: &PgPool,
    flag_id: Uuid,
    env_id: Uuid,
) -> Result<Vec<GroupRow>, AppError> {
    let flag_env =
        crate::domains::flag_environments::repo::find_by_flag_and_env(db, flag_id, env_id)
            .await?
            .ok_or_else(|| AppError::not_found("FlagEnvironment", None))?;

    repo::find_groups_for_flag_env(db, flag_env.id)
        .await
        .map_err(AppError::from)
}

pub async fn add_group_to_flag(
    db: &PgPool,
    redis: &fred::clients::RedisPool,
    flag_id: Uuid,
    env_id: Uuid,
    group_id: Uuid,
) -> Result<FlagGroupRow, AppError> {
    // Verify flag environment exists
    let flag_env =
        crate::domains::flag_environments::repo::find_by_flag_and_env(db, flag_id, env_id)
            .await?
            .ok_or_else(|| AppError::not_found("FlagEnvironment", None))?;

    // Verify group exists
    repo::find_by_id(db, group_id)
        .await?
        .ok_or_else(|| AppError::not_found("Group", Some(&group_id.to_string())))?;

    let result = repo::add_group_to_flag(db, flag_env.id, group_id).await?;

    // Invalidate cache
    let cache_key = format!("flick:flag_env:{}:{}", flag_id, env_id);
    let _: Result<(), _> = redis.del::<(), _>(&cache_key).await;

    Ok(result)
}

pub async fn remove_group_from_flag(
    db: &PgPool,
    redis: &fred::clients::RedisPool,
    flag_id: Uuid,
    env_id: Uuid,
    group_id: Uuid,
) -> Result<bool, AppError> {
    // Verify flag environment exists
    let flag_env =
        crate::domains::flag_environments::repo::find_by_flag_and_env(db, flag_id, env_id)
            .await?
            .ok_or_else(|| AppError::not_found("FlagEnvironment", None))?;

    let removed = repo::remove_group_from_flag(db, flag_env.id, group_id).await?;
    if !removed {
        return Err(AppError::not_found("FlagGroup", None));
    }

    // Invalidate cache
    let cache_key = format!("flick:flag_env:{}:{}", flag_id, env_id);
    let _: Result<(), _> = redis.del::<(), _>(&cache_key).await;

    Ok(removed)
}
