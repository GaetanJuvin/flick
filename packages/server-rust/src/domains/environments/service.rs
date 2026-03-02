use sqlx::PgPool;
use uuid::Uuid;

use crate::error::AppError;

use super::repo::{self, EnvironmentRow};

pub struct CreateEnvironmentInput {
    pub name: String,
    pub slug: String,
    pub color: String,
    pub sort_order: i32,
}

pub struct UpdateEnvironmentInput {
    pub name: Option<String>,
    pub color: Option<String>,
    pub sort_order: Option<i32>,
}

pub async fn list_environments(
    db: &PgPool,
    project_id: Uuid,
) -> Result<Vec<EnvironmentRow>, AppError> {
    repo::find_by_project(db, project_id)
        .await
        .map_err(AppError::from)
}

pub async fn get_environment(db: &PgPool, id: Uuid) -> Result<EnvironmentRow, AppError> {
    repo::find_by_id(db, id)
        .await?
        .ok_or_else(|| AppError::not_found("Environment", Some(&id.to_string())))
}

pub async fn create_environment(
    db: &PgPool,
    project_id: Uuid,
    input: CreateEnvironmentInput,
) -> Result<EnvironmentRow, AppError> {
    // Check for slug conflict within the project
    if repo::find_by_slug(db, project_id, &input.slug)
        .await?
        .is_some()
    {
        return Err(AppError::Conflict(format!(
            "Environment with slug '{}' already exists in this project",
            input.slug
        )));
    }

    repo::create(
        db,
        project_id,
        &input.name,
        &input.slug,
        &input.color,
        input.sort_order,
    )
    .await
    .map_err(AppError::from)
}

pub async fn update_environment(
    db: &PgPool,
    id: Uuid,
    input: UpdateEnvironmentInput,
) -> Result<EnvironmentRow, AppError> {
    repo::update(
        db,
        id,
        input.name.as_deref(),
        input.color.as_deref(),
        input.sort_order,
    )
    .await?
    .ok_or_else(|| AppError::not_found("Environment", Some(&id.to_string())))
}

pub async fn delete_environment(db: &PgPool, id: Uuid) -> Result<Uuid, AppError> {
    repo::remove(db, id)
        .await?
        .ok_or_else(|| AppError::not_found("Environment", Some(&id.to_string())))
}
