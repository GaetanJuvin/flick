use axum::{
    extract::{Path, State},
    routing::{get, post},
    Json, Router,
};
use serde::Deserialize;
use serde_json::{json, Value};
use uuid::Uuid;

use crate::app::AppState;
use crate::auth::AuthUser;
use crate::error::AppError;
use crate::middleware::rbac;

use super::service::{self, UpdateFlagEnvInput};

#[derive(Debug, Deserialize)]
pub struct UpdateFlagEnvBody {
    pub enabled: Option<bool>,
    pub gate_config: Option<Value>,
}

async fn list_flag_environments(
    _user: AuthUser,
    State(state): State<AppState>,
    Path((_project_id, flag_id)): Path<(Uuid, Uuid)>,
) -> Result<Json<serde_json::Value>, AppError> {
    let envs = service::get_flag_environments(&state.db, flag_id).await?;
    Ok(Json(json!({ "data": envs })))
}

async fn get_flag_environment(
    _user: AuthUser,
    State(state): State<AppState>,
    Path((_project_id, flag_id, env_id)): Path<(Uuid, Uuid, Uuid)>,
) -> Result<Json<serde_json::Value>, AppError> {
    let fe = service::get_flag_environment(&state.db, flag_id, env_id).await?;
    Ok(Json(json!({ "data": fe })))
}

async fn update_flag_environment(
    user: AuthUser,
    State(state): State<AppState>,
    Path((_project_id, flag_id, env_id)): Path<(Uuid, Uuid, Uuid)>,
    Json(body): Json<UpdateFlagEnvBody>,
) -> Result<Json<serde_json::Value>, AppError> {
    rbac::require_admin(&user)?;

    let input = UpdateFlagEnvInput {
        enabled: body.enabled,
        gate_config: body.gate_config,
    };

    let fe =
        service::update_flag_environment(&state.db, &state.redis, flag_id, env_id, input)
            .await?;
    Ok(Json(json!({ "data": fe })))
}

async fn toggle_flag_environment(
    user: AuthUser,
    State(state): State<AppState>,
    Path((_project_id, flag_id, env_id)): Path<(Uuid, Uuid, Uuid)>,
) -> Result<Json<serde_json::Value>, AppError> {
    rbac::require_admin(&user)?;

    let fe =
        service::toggle_flag_environment(&state.db, &state.redis, flag_id, env_id).await?;
    Ok(Json(json!({ "data": fe })))
}

pub fn router() -> Router<AppState> {
    Router::new()
        .route(
            "/projects/{projectId}/flags/{flagId}/environments",
            get(list_flag_environments),
        )
        .route(
            "/projects/{projectId}/flags/{flagId}/environments/{envId}",
            get(get_flag_environment).patch(update_flag_environment),
        )
        .route(
            "/projects/{projectId}/flags/{flagId}/environments/{envId}/toggle",
            post(toggle_flag_environment),
        )
}
