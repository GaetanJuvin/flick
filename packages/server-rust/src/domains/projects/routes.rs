use axum::{
    extract::{Path, State},
    routing::{get, patch, post},
    Json, Router,
};
use serde::Deserialize;
use serde_json::json;
use uuid::Uuid;

use crate::app::AppState;
use crate::auth::AuthUser;
use crate::error::AppError;
use crate::middleware::rbac;

use super::service;

#[derive(Debug, Deserialize)]
pub struct CreateProjectBody {
    pub name: String,
    pub slug: String,
}

#[derive(Debug, Deserialize)]
pub struct UpdateProjectBody {
    pub name: Option<String>,
}

async fn list_projects(
    _user: AuthUser,
    State(state): State<AppState>,
) -> Result<Json<serde_json::Value>, AppError> {
    let projects = service::list_projects(&state.db).await?;
    Ok(Json(json!({ "data": projects })))
}

async fn create_project(
    user: AuthUser,
    State(state): State<AppState>,
    Json(body): Json<CreateProjectBody>,
) -> Result<Json<serde_json::Value>, AppError> {
    rbac::require_admin(&user)?;
    let project = service::create_project(&state.db, &body.name, &body.slug).await?;
    Ok(Json(json!({ "data": project })))
}

async fn get_project(
    _user: AuthUser,
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
) -> Result<Json<serde_json::Value>, AppError> {
    let project = service::get_project(&state.db, id).await?;
    Ok(Json(json!({ "data": project })))
}

async fn update_project(
    user: AuthUser,
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
    Json(body): Json<UpdateProjectBody>,
) -> Result<Json<serde_json::Value>, AppError> {
    rbac::require_admin(&user)?;
    let project = service::update_project(&state.db, id, body.name.as_deref()).await?;
    Ok(Json(json!({ "data": project })))
}

pub fn router() -> Router<AppState> {
    Router::new()
        .route("/projects", get(list_projects).post(create_project))
        .route("/projects/:id", get(get_project).patch(update_project))
}
