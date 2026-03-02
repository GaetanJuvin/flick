use axum::extract::{Path, Query, State};
use axum::routing::get;
use axum::{Json, Router};
use serde::Deserialize;
use serde_json::json;
use uuid::Uuid;

use crate::app::AppState;
use crate::auth::AuthUser;
use crate::error::AppError;

use super::service;

#[derive(Debug, Deserialize)]
struct AuditQueryParams {
    entity_type: Option<String>,
    entity_id: Option<String>,
    actor_id: Option<String>,
    action: Option<String>,
    cursor: Option<String>,
    limit: Option<i64>,
}

// GET /projects/{projectId}/audit
async fn get_audit_log(
    State(state): State<AppState>,
    user: AuthUser,
    Path(project_id): Path<Uuid>,
    Query(params): Query<AuditQueryParams>,
) -> Result<Json<serde_json::Value>, AppError> {
    let _ = &user;

    let actor_id = params
        .actor_id
        .as_deref()
        .map(|s| s.parse::<Uuid>())
        .transpose()
        .map_err(|_| AppError::validation("Invalid actor_id"))?;

    let (data, cursor, has_more) = service::get_audit_log(
        &state.db,
        project_id,
        params.entity_type.as_deref(),
        params.entity_id.as_deref(),
        actor_id,
        params.action.as_deref(),
        params.cursor.as_deref(),
        params.limit,
    )
    .await?;

    Ok(Json(json!({
        "data": data,
        "cursor": cursor,
        "has_more": has_more,
    })))
}

pub fn router() -> Router<AppState> {
    Router::new().route("/projects/{projectId}/audit", get(get_audit_log))
}
