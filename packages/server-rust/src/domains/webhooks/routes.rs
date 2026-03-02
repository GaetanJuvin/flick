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
use crate::middleware::rbac::require_admin;

use super::service;

// ── Request bodies ─────────────────────────────────────────────────────────

#[derive(Debug, Deserialize)]
pub struct CreateWebhookBody {
    pub url: String,
    pub events: Vec<String>,
}

#[derive(Debug, Deserialize)]
pub struct UpdateWebhookBody {
    pub url: Option<String>,
    pub events: Option<Vec<String>>,
    pub status: Option<String>,
}

// ── Handlers ───────────────────────────────────────────────────────────────

async fn list_webhooks(
    State(state): State<AppState>,
    user: AuthUser,
    Path(project_id): Path<Uuid>,
) -> Result<Json<serde_json::Value>, AppError> {
    let _ = user;
    let webhooks = service::list_webhooks(&state.db, project_id).await?;
    Ok(Json(json!({ "data": webhooks })))
}

async fn create_webhook(
    State(state): State<AppState>,
    user: AuthUser,
    Path(project_id): Path<Uuid>,
    Json(body): Json<CreateWebhookBody>,
) -> Result<Json<serde_json::Value>, AppError> {
    require_admin(&user)?;
    let webhook = service::create_webhook(&state.db, project_id, &body.url, &body.events).await?;
    Ok(Json(json!({ "data": webhook })))
}

async fn update_webhook(
    State(state): State<AppState>,
    user: AuthUser,
    Path((_project_id, id)): Path<(Uuid, Uuid)>,
    Json(body): Json<UpdateWebhookBody>,
) -> Result<Json<serde_json::Value>, AppError> {
    require_admin(&user)?;
    let input = service::UpdateWebhookInput {
        url: body.url,
        events: body.events,
        status: body.status,
    };
    let webhook = service::update_webhook(&state.db, id, input).await?;
    Ok(Json(json!({ "data": webhook })))
}

async fn delete_webhook(
    State(state): State<AppState>,
    user: AuthUser,
    Path((_project_id, id)): Path<(Uuid, Uuid)>,
) -> Result<Json<serde_json::Value>, AppError> {
    require_admin(&user)?;
    service::delete_webhook(&state.db, id).await?;
    Ok(Json(json!({ "success": true })))
}

async fn test_webhook(
    State(state): State<AppState>,
    user: AuthUser,
    Path((_project_id, id)): Path<(Uuid, Uuid)>,
) -> Result<Json<serde_json::Value>, AppError> {
    require_admin(&user)?;
    let delivery = service::test_webhook(&state.db, id).await?;
    Ok(Json(json!({ "data": delivery })))
}

async fn list_deliveries(
    State(state): State<AppState>,
    user: AuthUser,
    Path((_project_id, id)): Path<(Uuid, Uuid)>,
) -> Result<Json<serde_json::Value>, AppError> {
    let _ = user;
    let deliveries = service::get_deliveries(&state.db, id).await?;
    Ok(Json(json!({ "data": deliveries })))
}

// ── Router ─────────────────────────────────────────────────────────────────

pub fn router() -> Router<AppState> {
    Router::new()
        .route("/projects/{projectId}/webhooks", get(list_webhooks))
        .route("/projects/{projectId}/webhooks", post(create_webhook))
        .route(
            "/projects/{projectId}/webhooks/{id}",
            patch(update_webhook),
        )
        .route(
            "/projects/{projectId}/webhooks/{id}",
            delete(delete_webhook),
        )
        .route(
            "/projects/{projectId}/webhooks/{id}/test",
            post(test_webhook),
        )
        .route(
            "/projects/{projectId}/webhooks/{id}/deliveries",
            get(list_deliveries),
        )
}
