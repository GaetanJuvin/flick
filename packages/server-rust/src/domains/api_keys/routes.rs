use axum::extract::{Path, State};
use axum::routing::{delete, get, post};
use axum::{Json, Router};
use serde::Deserialize;
use serde_json::json;
use uuid::Uuid;

use crate::app::AppState;
use crate::auth::AuthUser;
use crate::error::AppError;
use crate::middleware::rbac::require_admin;

use super::service;

#[derive(Debug, Deserialize)]
struct CreateApiKeyBody {
    name: String,
    #[serde(rename = "type")]
    key_type: String,
    environment_id: Option<Uuid>,
}

// GET /projects/{projectId}/api-keys
async fn list_api_keys(
    State(state): State<AppState>,
    user: AuthUser,
    Path(project_id): Path<Uuid>,
) -> Result<Json<serde_json::Value>, AppError> {
    require_admin(&user)?;
    let keys = service::list_api_keys(&state.db, project_id).await?;
    Ok(Json(json!({ "data": keys })))
}

// POST /projects/{projectId}/api-keys
async fn create_api_key(
    State(state): State<AppState>,
    user: AuthUser,
    Path(project_id): Path<Uuid>,
    Json(body): Json<CreateApiKeyBody>,
) -> Result<Json<serde_json::Value>, AppError> {
    require_admin(&user)?;
    let created = service::create_api_key(
        &state.db,
        project_id,
        &body.name,
        &body.key_type,
        body.environment_id,
        &user,
    )
    .await?;
    Ok(Json(json!({ "data": created })))
}

// DELETE /projects/{projectId}/api-keys/{id}
async fn revoke_api_key(
    State(state): State<AppState>,
    user: AuthUser,
    Path((_project_id, id)): Path<(Uuid, Uuid)>,
) -> Result<Json<serde_json::Value>, AppError> {
    require_admin(&user)?;
    service::revoke_api_key(&state.db, &state.redis, id).await?;
    Ok(Json(json!({ "data": { "deleted": true } })))
}

pub fn router() -> Router<AppState> {
    Router::new()
        .route(
            "/projects/{projectId}/api-keys",
            get(list_api_keys).post(create_api_key),
        )
        .route(
            "/projects/{projectId}/api-keys/{id}",
            delete(revoke_api_key),
        )
}
