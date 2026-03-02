use sqlx::PgPool;
use uuid::Uuid;

use crate::error::AppError;

use super::repo::{self, FlagRow};

pub async fn list_flags(
    db: &PgPool,
    project_id: Uuid,
    archived: Option<bool>,
    tags: Option<Vec<String>>,
) -> Result<Vec<FlagRow>, AppError> {
    repo::find_by_project(db, project_id, archived, tags)
        .await
        .map_err(AppError::from)
}

pub async fn get_flag(db: &PgPool, id: Uuid) -> Result<FlagRow, AppError> {
    repo::find_by_id(db, id)
        .await?
        .ok_or_else(|| AppError::not_found("Flag", Some(&id.to_string())))
}

pub struct CreateFlagInput<'a> {
    pub key: &'a str,
    pub name: &'a str,
    pub description: &'a str,
    pub gate_type: &'a str,
    pub tags: &'a [String],
}

pub async fn create_flag(
    db: &PgPool,
    project_id: Uuid,
    input: CreateFlagInput<'_>,
) -> Result<FlagRow, AppError> {
    // Check for key conflict within project
    if repo::find_by_key(db, project_id, input.key).await?.is_some() {
        return Err(AppError::conflict(format!(
            "Flag with key '{}' already exists in this project",
            input.key
        )));
    }

    repo::create(
        db,
        project_id,
        input.key,
        input.name,
        input.description,
        input.gate_type,
        input.tags,
    )
    .await
}

pub struct UpdateFlagInput {
    pub name: Option<String>,
    pub description: Option<String>,
    pub tags: Option<Vec<String>>,
}

pub async fn update_flag(
    db: &PgPool,
    id: Uuid,
    input: UpdateFlagInput,
) -> Result<FlagRow, AppError> {
    repo::update(
        db,
        id,
        input.name.as_deref(),
        input.description.as_deref(),
        input.tags.as_deref(),
    )
    .await?
    .ok_or_else(|| AppError::not_found("Flag", Some(&id.to_string())))
}

pub async fn archive_flag(db: &PgPool, id: Uuid) -> Result<FlagRow, AppError> {
    repo::archive(db, id)
        .await?
        .ok_or_else(|| AppError::not_found("Flag", Some(&id.to_string())))
}

pub async fn restore_flag(db: &PgPool, id: Uuid) -> Result<FlagRow, AppError> {
    repo::restore(db, id)
        .await?
        .ok_or_else(|| AppError::not_found("Flag", Some(&id.to_string())))
}

pub async fn delete_flag(db: &PgPool, id: Uuid) -> Result<Uuid, AppError> {
    repo::remove(db, id)
        .await?
        .ok_or_else(|| AppError::not_found("Flag", Some(&id.to_string())))
}
