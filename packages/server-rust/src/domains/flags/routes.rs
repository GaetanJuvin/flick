use axum::{
    extract::{Path, Query, State},
    routing::{get, post},
    Json, Router,
};
use serde::Deserialize;
use serde_json::json;
use uuid::Uuid;

use crate::app::AppState;
use crate::auth::AuthUser;
use crate::error::AppError;
use crate::middleware::rbac;

use super::service::{self, CreateFlagInput, UpdateFlagInput};

#[derive(Debug, Deserialize)]
pub struct ListFlagsQuery {
    pub archived: Option<bool>,
    pub tags: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct CreateFlagBody {
    pub key: String,
    pub name: String,
    #[serde(default)]
    pub description: String,
    #[serde(default = "default_gate_type")]
    pub gate_type: String,
    #[serde(default)]
    pub tags: Vec<String>,
}

fn default_gate_type() -> String {
    "boolean".to_string()
}

#[derive(Debug, Deserialize)]
pub struct UpdateFlagBody {
    pub name: Option<String>,
    pub description: Option<String>,
    pub tags: Option<Vec<String>>,
}

async fn list_flags(
    _user: AuthUser,
    State(state): State<AppState>,
    Path(project_id): Path<Uuid>,
    Query(query): Query<ListFlagsQuery>,
) -> Result<Json<serde_json::Value>, AppError> {
    let tags = query
        .tags
        .map(|t| t.split(',').map(|s| s.trim().to_string()).collect());

    let flags = service::list_flags(&state.db, project_id, query.archived, tags).await?;
    Ok(Json(json!({ "data": flags })))
}

async fn create_flag(
    user: AuthUser,
    State(state): State<AppState>,
    Path(project_id): Path<Uuid>,
    Json(body): Json<CreateFlagBody>,
) -> Result<Json<serde_json::Value>, AppError> {
    rbac::require_admin(&user)?;

    let input = CreateFlagInput {
        key: &body.key,
        name: &body.name,
        description: &body.description,
        gate_type: &body.gate_type,
        tags: &body.tags,
    };

    let flag = service::create_flag(&state.db, project_id, input).await?;
    Ok(Json(json!({ "data": flag })))
}

async fn get_flag(
    _user: AuthUser,
    State(state): State<AppState>,
    Path((_project_id, id)): Path<(Uuid, Uuid)>,
) -> Result<Json<serde_json::Value>, AppError> {
    let flag = service::get_flag(&state.db, id).await?;
    Ok(Json(json!({ "data": flag })))
}

async fn update_flag(
    user: AuthUser,
    State(state): State<AppState>,
    Path((_project_id, id)): Path<(Uuid, Uuid)>,
    Json(body): Json<UpdateFlagBody>,
) -> Result<Json<serde_json::Value>, AppError> {
    rbac::require_admin(&user)?;

    let input = UpdateFlagInput {
        name: body.name,
        description: body.description,
        tags: body.tags,
    };

    let flag = service::update_flag(&state.db, id, input).await?;
    Ok(Json(json!({ "data": flag })))
}

async fn delete_flag(
    user: AuthUser,
    State(state): State<AppState>,
    Path((_project_id, id)): Path<(Uuid, Uuid)>,
) -> Result<Json<serde_json::Value>, AppError> {
    rbac::require_admin(&user)?;
    let deleted_id = service::delete_flag(&state.db, id).await?;
    Ok(Json(json!({ "data": { "id": deleted_id } })))
}

async fn archive_flag(
    user: AuthUser,
    State(state): State<AppState>,
    Path((_project_id, id)): Path<(Uuid, Uuid)>,
) -> Result<Json<serde_json::Value>, AppError> {
    rbac::require_admin(&user)?;
    let flag = service::archive_flag(&state.db, id).await?;
    Ok(Json(json!({ "data": flag })))
}

async fn restore_flag(
    user: AuthUser,
    State(state): State<AppState>,
    Path((_project_id, id)): Path<(Uuid, Uuid)>,
) -> Result<Json<serde_json::Value>, AppError> {
    rbac::require_admin(&user)?;
    let flag = service::restore_flag(&state.db, id).await?;
    Ok(Json(json!({ "data": flag })))
}

pub fn router() -> Router<AppState> {
    Router::new()
        .route(
            "/projects/:projectId/flags",
            get(list_flags).post(create_flag),
        )
        .route(
            "/projects/:projectId/flags/:id",
            get(get_flag).patch(update_flag).delete(delete_flag),
        )
        .route(
            "/projects/:projectId/flags/:id/archive",
            post(archive_flag),
        )
        .route(
            "/projects/:projectId/flags/:id/restore",
            post(restore_flag),
        )
}
