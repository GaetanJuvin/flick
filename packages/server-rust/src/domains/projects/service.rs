use sqlx::PgPool;
use uuid::Uuid;

use crate::error::AppError;

use super::repo::{self, ProjectRow};
use crate::domains::environments;

pub async fn list_projects(db: &PgPool) -> Result<Vec<ProjectRow>, AppError> {
    repo::find_all(db).await.map_err(AppError::from)
}

pub async fn get_project(db: &PgPool, id: Uuid) -> Result<ProjectRow, AppError> {
    repo::find_by_id(db, id)
        .await?
        .ok_or_else(|| AppError::not_found("Project", Some(&id.to_string())))
}

pub async fn create_project(
    db: &PgPool,
    name: &str,
    slug: &str,
) -> Result<ProjectRow, AppError> {
    // Check for slug conflict
    if repo::find_by_slug(db, slug).await?.is_some() {
        return Err(AppError::Conflict(format!(
            "Project with slug '{slug}' already exists"
        )));
    }

    let project = repo::create(db, name, slug).await?;

    // Create default environments
    let defaults = [
        ("Development", "dev", "#22c55e", 0),
        ("Staging", "staging", "#f59e0b", 1),
        ("Production", "prod", "#ef4444", 2),
    ];

    for (env_name, env_slug, color, sort_order) in defaults {
        environments::repo::create(db, project.id, env_name, env_slug, color, sort_order).await?;
    }

    Ok(project)
}

pub async fn update_project(
    db: &PgPool,
    id: Uuid,
    name: Option<&str>,
) -> Result<ProjectRow, AppError> {
    repo::update(db, id, name)
        .await?
        .ok_or_else(|| AppError::not_found("Project", Some(&id.to_string())))
}
