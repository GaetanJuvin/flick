use std::collections::HashMap;

use axum::extract::{Query, State};
use axum::http::{HeaderMap, StatusCode};
use axum::middleware;
use axum::response::{IntoResponse, Response};
use axum::routing::{get, post};
use axum::{Json, Router};
use serde::Deserialize;

use crate::app::AppState;
use crate::auth::SdkAuth;
use crate::error::AppError;
use crate::evaluation::{EvaluationContext, EvaluationResult};
use crate::middleware::rate_limit::evaluate_rate_limit;

use super::service;

// ---------------------------------------------------------------------------
// Request / query types
// ---------------------------------------------------------------------------

#[derive(Debug, Deserialize)]
pub struct EvaluateRequest {
    pub flag_key: String,
    pub context: EvaluationContext,
}

#[derive(Debug, Deserialize)]
pub struct BatchEvaluateRequest {
    pub context: EvaluationContext,
}

#[derive(Debug, Deserialize)]
pub struct EnvIdQuery {
    pub environment_id: Option<String>,
}

// ---------------------------------------------------------------------------
// Resolve environment ID from header, query param, or error
// ---------------------------------------------------------------------------

fn resolve_env_id(
    headers: &HeaderMap,
    query: &EnvIdQuery,
    _auth: &SdkAuth,
) -> Result<String, AppError> {
    // 1. X-Environment-Id header
    if let Some(header_val) = headers.get("X-Environment-Id") {
        if let Ok(s) = header_val.to_str() {
            if !s.is_empty() {
                return Ok(s.to_string());
            }
        }
    }

    // 2. Query param ?environment_id=...
    if let Some(ref env_id) = query.environment_id {
        if !env_id.is_empty() {
            return Ok(env_id.clone());
        }
    }

    // 3. No environment ID found
    Err(AppError::validation(
        "Environment ID required. Set X-Environment-Id header or pass ?environment_id= query param.",
    ))
}

// ---------------------------------------------------------------------------
// POST /evaluate
// ---------------------------------------------------------------------------

async fn evaluate_handler(
    State(state): State<AppState>,
    auth: SdkAuth,
    headers: HeaderMap,
    Query(query): Query<EnvIdQuery>,
    Json(body): Json<EvaluateRequest>,
) -> Result<Json<serde_json::Value>, AppError> {
    let env_id = resolve_env_id(&headers, &query, &auth)?;
    let result = service::evaluate(&state.db, &state.redis, &env_id, &body.flag_key, &body.context)
        .await
        .map_err(|e| AppError::Internal(e.into()))?;

    Ok(Json(serde_json::json!({ "data": result })))
}

// ---------------------------------------------------------------------------
// POST /evaluate/batch
// ---------------------------------------------------------------------------

async fn batch_evaluate_handler(
    State(state): State<AppState>,
    auth: SdkAuth,
    headers: HeaderMap,
    Query(query): Query<EnvIdQuery>,
    Json(body): Json<BatchEvaluateRequest>,
) -> Result<Json<serde_json::Value>, AppError> {
    let env_id = resolve_env_id(&headers, &query, &auth)?;
    let results: HashMap<String, EvaluationResult> =
        service::evaluate_batch(&state.db, &state.redis, &env_id, &body.context)
            .await
            .map_err(|e| AppError::Internal(e.into()))?;

    Ok(Json(serde_json::json!({ "data": { "flags": results } })))
}

// ---------------------------------------------------------------------------
// GET /evaluate/config
// ---------------------------------------------------------------------------

async fn config_handler(
    State(state): State<AppState>,
    auth: SdkAuth,
    headers: HeaderMap,
    Query(query): Query<EnvIdQuery>,
) -> Result<Response, AppError> {
    let env_id = resolve_env_id(&headers, &query, &auth)?;
    let config = service::get_full_config(&state.db, &state.redis, &env_id)
        .await
        .map_err(|e| AppError::Internal(e.into()))?;

    // ETag support for SDK polling
    let etag = format!("\"{}\"", config.version);

    // Check If-None-Match → 304 Not Modified
    if let Some(if_none_match) = headers.get("If-None-Match") {
        if let Ok(val) = if_none_match.to_str() {
            if val == etag {
                return Ok(StatusCode::NOT_MODIFIED.into_response());
            }
        }
    }

    let body = Json(serde_json::json!({ "data": config }));
    let mut response = body.into_response();
    response
        .headers_mut()
        .insert("ETag", etag.parse().unwrap());

    Ok(response)
}

// ---------------------------------------------------------------------------
// Router
// ---------------------------------------------------------------------------

pub fn router(state: AppState) -> Router<AppState> {
    Router::new()
        .route("/evaluate", post(evaluate_handler))
        .route("/evaluate/batch", post(batch_evaluate_handler))
        .route("/evaluate/config", get(config_handler))
        .route_layer(middleware::from_fn_with_state(
            state,
            evaluate_rate_limit,
        ))
}
