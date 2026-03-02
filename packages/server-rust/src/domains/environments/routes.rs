use axum::{
    extract::{Path, State},
    routing::{delete, get, patch, post},
    Json, Router,
};
use serde::Deserialize;
use serde_json::json;
use uuid::Uuid;

use crate::app::AppState;
use crate::auth::AuthUser;
use crate::error::AppError;
use crate::middleware::rbac;

use super::service::{self, CreateEnvironmentInput, UpdateEnvironmentInput};

#[derive(Debug, Deserialize)]
pub struct CreateEnvironmentBody {
    pub name: String,
    pub slug: String,
    pub color: String,
    pub sort_order: i32,
}

#[derive(Debug, Deserialize)]
pub struct UpdateEnvironmentBody {
    pub name: Option<String>,
    pub color: Option<String>,
    pub sort_order: Option<i32>,
}

async fn list_environments(
    _user: AuthUser,
    State(state): State<AppState>,
    Path(project_id): Path<Uuid>,
) -> Result<Json<serde_json::Value>, AppError> {
    let environments = service::list_environments(&state.db, project_id).await?;
    Ok(Json(json!({ "data": environments })))
}

async fn create_environment(
    user: AuthUser,
    State(state): State<AppState>,
    Path(project_id): Path<Uuid>,
    Json(body): Json<CreateEnvironmentBody>,
) -> Result<Json<serde_json::Value>, AppError> {
    rbac::require_admin(&user)?;
    let environment = service::create_environment(
        &state.db,
        project_id,
        CreateEnvironmentInput {
            name: body.name,
            slug: body.slug,
            color: body.color,
            sort_order: body.sort_order,
        },
    )
    .await?;
    Ok(Json(json!({ "data": environment })))
}

async fn update_environment(
    user: AuthUser,
    State(state): State<AppState>,
    Path((_project_id, id)): Path<(Uuid, Uuid)>,
    Json(body): Json<UpdateEnvironmentBody>,
) -> Result<Json<serde_json::Value>, AppError> {
    rbac::require_admin(&user)?;
    let environment = service::update_environment(
        &state.db,
        id,
        UpdateEnvironmentInput {
            name: body.name,
            color: body.color,
            sort_order: body.sort_order,
        },
    )
    .await?;
    Ok(Json(json!({ "data": environment })))
}

async fn delete_environment(
    user: AuthUser,
    State(state): State<AppState>,
    Path((_project_id, id)): Path<(Uuid, Uuid)>,
) -> Result<Json<serde_json::Value>, AppError> {
    rbac::require_admin(&user)?;
    let deleted_id = service::delete_environment(&state.db, id).await?;
    Ok(Json(json!({ "data": { "id": deleted_id } })))
}

pub fn router() -> Router<AppState> {
    Router::new()
        .route(
            "/projects/{projectId}/environments",
            get(list_environments).post(create_environment),
        )
        .route(
            "/projects/{projectId}/environments/{id}",
            patch(update_environment).delete(delete_environment),
        )
}
