use axum::extract::{Path, State};
use axum::routing::{delete, get, patch, post};
use axum::{Json, Router};
use serde::Deserialize;
use serde_json::{json, Value};
use uuid::Uuid;

use crate::app::AppState;
use crate::auth::AuthUser;
use crate::error::AppError;
use crate::middleware::rbac::require_admin;

use super::service;

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct CreateGroupBody {
    name: String,
    slug: String,
    #[serde(default)]
    description: Option<String>,
    #[serde(default)]
    rules: Option<Value>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct UpdateGroupBody {
    name: Option<String>,
    description: Option<String>,
    rules: Option<Value>,
}

#[derive(Debug, Deserialize)]
struct AddGroupBody {
    group_id: String,
}

// GET /projects/:projectId/groups
async fn list_groups(
    State(state): State<AppState>,
    user: AuthUser,
    Path(project_id): Path<Uuid>,
) -> Result<Json<Value>, AppError> {
    let _ = &user;
    let groups = service::list_groups(&state.db, project_id).await?;
    Ok(Json(json!({ "data": groups })))
}

// POST /projects/:projectId/groups
async fn create_group(
    State(state): State<AppState>,
    user: AuthUser,
    Path(project_id): Path<Uuid>,
    Json(body): Json<CreateGroupBody>,
) -> Result<Json<Value>, AppError> {
    require_admin(&user)?;
    let rules = body.rules.unwrap_or(json!([]));
    let description = body.description.unwrap_or_default();
    let group =
        service::create_group(&state.db, project_id, &body.name, &body.slug, &description, rules)
            .await?;
    Ok(Json(json!({ "data": group })))
}

// GET /projects/:projectId/groups/:id
async fn get_group(
    State(state): State<AppState>,
    user: AuthUser,
    Path((_project_id, id)): Path<(Uuid, Uuid)>,
) -> Result<Json<Value>, AppError> {
    let _ = &user;
    let group = service::get_group(&state.db, id).await?;
    Ok(Json(json!({ "data": group })))
}

// PATCH /projects/:projectId/groups/:id
async fn update_group(
    State(state): State<AppState>,
    user: AuthUser,
    Path((_project_id, id)): Path<(Uuid, Uuid)>,
    Json(body): Json<UpdateGroupBody>,
) -> Result<Json<Value>, AppError> {
    require_admin(&user)?;
    let group = service::update_group(
        &state.db,
        id,
        body.name.as_deref(),
        body.description.as_deref(),
        body.rules,
    )
    .await?;
    Ok(Json(json!({ "data": group })))
}

// DELETE /projects/:projectId/groups/:id
async fn delete_group(
    State(state): State<AppState>,
    user: AuthUser,
    Path((_project_id, id)): Path<(Uuid, Uuid)>,
) -> Result<Json<Value>, AppError> {
    require_admin(&user)?;
    service::delete_group(&state.db, id).await?;
    Ok(Json(json!({ "data": { "deleted": true } })))
}

// GET /projects/:projectId/flags/:flagId/environments/:envId/groups
async fn get_flag_groups(
    State(state): State<AppState>,
    user: AuthUser,
    Path((_project_id, flag_id, env_id)): Path<(Uuid, Uuid, Uuid)>,
) -> Result<Json<Value>, AppError> {
    let _ = &user;
    let groups = service::get_groups_for_flag_env(&state.db, flag_id, env_id).await?;
    Ok(Json(json!({ "data": groups })))
}

// POST /projects/:projectId/flags/:flagId/environments/:envId/groups
async fn add_group_to_flag(
    State(state): State<AppState>,
    user: AuthUser,
    Path((_project_id, flag_id, env_id)): Path<(Uuid, Uuid, Uuid)>,
    Json(body): Json<AddGroupBody>,
) -> Result<Json<Value>, AppError> {
    require_admin(&user)?;
    let group_id: Uuid = body
        .group_id
        .parse()
        .map_err(|_| AppError::validation("Invalid group_id"))?;
    let flag_group =
        service::add_group_to_flag(&state.db, &state.redis, flag_id, env_id, group_id).await?;
    Ok(Json(json!({ "data": flag_group })))
}

// DELETE /projects/:projectId/flags/:flagId/environments/:envId/groups/:groupId
async fn remove_group_from_flag(
    State(state): State<AppState>,
    user: AuthUser,
    Path((_project_id, flag_id, env_id, group_id)): Path<(Uuid, Uuid, Uuid, Uuid)>,
) -> Result<Json<Value>, AppError> {
    require_admin(&user)?;
    service::remove_group_from_flag(&state.db, &state.redis, flag_id, env_id, group_id).await?;
    Ok(Json(json!({ "data": { "deleted": true } })))
}

pub fn router() -> Router<AppState> {
    Router::new()
        // Group CRUD
        .route("/projects/:projectId/groups", get(list_groups).post(create_group))
        .route(
            "/projects/:projectId/groups/:id",
            get(get_group).patch(update_group).delete(delete_group),
        )
        // Flag-group associations
        .route(
            "/projects/:projectId/flags/:flagId/environments/:envId/groups",
            get(get_flag_groups).post(add_group_to_flag),
        )
        .route(
            "/projects/:projectId/flags/:flagId/environments/:envId/groups/:groupId",
            delete(remove_group_from_flag),
        )
}
