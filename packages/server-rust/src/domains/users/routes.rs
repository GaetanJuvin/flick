use axum::{
    extract::{Path, State},
    http::StatusCode,
    response::IntoResponse,
    routing::{get, patch, post},
    Json, Router,
};
use serde::Deserialize;
use serde_json::json;
use tower_cookies::{Cookie, Cookies};
use uuid::Uuid;

use crate::app::AppState;
use crate::auth::AuthUser;
use crate::config::AuthMode;
use crate::error::AppError;
use crate::middleware::rbac::require_admin;
use crate::middleware::rate_limit;

use super::service;

// ── Request bodies ─────────────────────────────────────────────────────────

#[derive(Debug, Deserialize)]
pub struct LoginBody {
    pub email: String,
    pub password: String,
}

#[derive(Debug, Deserialize)]
pub struct CreateUserBody {
    pub email: String,
    pub name: String,
    pub password: String,
    pub role: String,
}

#[derive(Debug, Deserialize)]
pub struct UpdateUserBody {
    pub name: Option<String>,
    pub email: Option<String>,
    pub role: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct UpdateProfileBody {
    pub name: Option<String>,
    pub email: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct ChangePasswordBody {
    pub current_password: String,
    pub new_password: String,
}

#[derive(Debug, Deserialize)]
pub struct ResetPasswordBody {
    pub password: String,
}

// ── Auth handlers ──────────────────────────────────────────────────────────

async fn auth_config(
    State(state): State<AppState>,
) -> Result<Json<serde_json::Value>, AppError> {
    let saml_enabled = matches!(state.config.auth_mode, AuthMode::Saml | AuthMode::Both);
    let auth_mode = match state.config.auth_mode {
        AuthMode::Password => "password",
        AuthMode::Saml => "saml",
        AuthMode::Both => "both",
    };
    Ok(Json(json!({
        "auth_mode": auth_mode,
        "saml_enabled": saml_enabled
    })))
}

async fn login(
    State(state): State<AppState>,
    cookies: Cookies,
    Json(body): Json<LoginBody>,
) -> Result<Json<serde_json::Value>, AppError> {
    // Rate limiting on login
    rate_limit::check_rate_limit(
        &state.redis,
        &rate_limit::LOGIN_EMAIL_LIMIT,
        &body.email,
    )
    .await?;

    let session_user =
        service::login(&state.db, &state.config, &body.email, &body.password).await?;

    // Build an AuthUser to create the session cookie
    let auth_user = AuthUser {
        id: session_user.id.to_string(),
        email: session_user.email.clone(),
        name: session_user.name.clone(),
        role: session_user.role.clone(),
        auth_method: session_user.auth_method.clone(),
        project_id: None,
        api_key_type: None,
    };

    let cookie_value = crate::auth::session::create_session_cookie(&auth_user);
    let mut cookie = Cookie::new("session", cookie_value);
    cookie.set_path("/");
    cookie.set_http_only(true);
    cookies.add(cookie);

    Ok(Json(json!({ "data": session_user })))
}

async fn logout(cookies: Cookies) -> Result<Json<serde_json::Value>, AppError> {
    cookies.remove(Cookie::from("session"));
    Ok(Json(json!({ "success": true })))
}

async fn me(
    State(state): State<AppState>,
    user: AuthUser,
) -> Result<Json<serde_json::Value>, AppError> {
    let user_id: Uuid = user.id.parse().map_err(|_| AppError::validation("Invalid user ID"))?;
    let profile = service::get_profile(&state.db, user_id).await?;
    Ok(Json(json!({ "data": profile })))
}

async fn saml_login() -> impl IntoResponse {
    (
        StatusCode::NOT_IMPLEMENTED,
        Json(json!({
            "error": {
                "code": "NOT_IMPLEMENTED",
                "message": "SAML login not yet implemented"
            }
        })),
    )
}

async fn saml_callback() -> impl IntoResponse {
    (
        StatusCode::NOT_IMPLEMENTED,
        Json(json!({
            "error": {
                "code": "NOT_IMPLEMENTED",
                "message": "SAML callback not yet implemented"
            }
        })),
    )
}

// ── Profile handlers ───────────────────────────────────────────────────────

async fn get_profile(
    State(state): State<AppState>,
    user: AuthUser,
) -> Result<Json<serde_json::Value>, AppError> {
    let user_id: Uuid = user.id.parse().map_err(|_| AppError::validation("Invalid user ID"))?;
    let profile = service::get_profile(&state.db, user_id).await?;
    Ok(Json(json!({ "data": profile })))
}

async fn update_profile(
    State(state): State<AppState>,
    user: AuthUser,
    Json(body): Json<UpdateProfileBody>,
) -> Result<Json<serde_json::Value>, AppError> {
    let user_id: Uuid = user.id.parse().map_err(|_| AppError::validation("Invalid user ID"))?;
    let input = service::UpdateProfileInput {
        name: body.name,
        email: body.email,
    };
    let profile = service::update_profile(&state.db, user_id, input).await?;
    Ok(Json(json!({ "data": profile })))
}

async fn change_password(
    State(state): State<AppState>,
    user: AuthUser,
    Json(body): Json<ChangePasswordBody>,
) -> Result<Json<serde_json::Value>, AppError> {
    let user_id: Uuid = user.id.parse().map_err(|_| AppError::validation("Invalid user ID"))?;
    service::change_password(
        &state.db,
        user_id,
        &body.current_password,
        &body.new_password,
    )
    .await?;
    Ok(Json(json!({ "success": true })))
}

// ── User management handlers ───────────────────────────────────────────────

async fn list_users(
    State(state): State<AppState>,
    user: AuthUser,
) -> Result<Json<serde_json::Value>, AppError> {
    require_admin(&user)?;
    let users = service::list_users(&state.db).await?;
    Ok(Json(json!({ "data": users })))
}

async fn create_user(
    State(state): State<AppState>,
    user: AuthUser,
    Json(body): Json<CreateUserBody>,
) -> Result<Json<serde_json::Value>, AppError> {
    require_admin(&user)?;
    let input = service::CreateUserInput {
        email: body.email,
        name: body.name,
        password: body.password,
        role: body.role,
    };
    let new_user = service::create_user(&state.db, &state.config, input).await?;
    Ok(Json(json!({ "data": new_user })))
}

async fn get_user(
    State(state): State<AppState>,
    user: AuthUser,
    Path(id): Path<Uuid>,
) -> Result<Json<serde_json::Value>, AppError> {
    let _ = user;
    let target = service::get_user(&state.db, id).await?;
    Ok(Json(json!({ "data": target })))
}

async fn update_user(
    State(state): State<AppState>,
    user: AuthUser,
    Path(id): Path<Uuid>,
    Json(body): Json<UpdateUserBody>,
) -> Result<Json<serde_json::Value>, AppError> {
    require_admin(&user)?;
    let input = service::UpdateUserInput {
        name: body.name,
        email: body.email,
        role: body.role,
    };
    let updated = service::update_user(&state.db, id, input).await?;
    Ok(Json(json!({ "data": updated })))
}

async fn delete_user(
    State(state): State<AppState>,
    user: AuthUser,
    Path(id): Path<Uuid>,
) -> Result<Json<serde_json::Value>, AppError> {
    require_admin(&user)?;
    service::delete_user(&state.db, id).await?;
    Ok(Json(json!({ "success": true })))
}

async fn reset_password(
    State(state): State<AppState>,
    user: AuthUser,
    Path(id): Path<Uuid>,
    Json(body): Json<ResetPasswordBody>,
) -> Result<Json<serde_json::Value>, AppError> {
    require_admin(&user)?;
    service::admin_reset_password(&state.db, id, &body.password).await?;
    Ok(Json(json!({ "success": true })))
}

// ── Router ─────────────────────────────────────────────────────────────────

pub fn router(state: AppState) -> Router<AppState> {
    Router::new()
        // Auth routes (no auth required for login/logout/config)
        .route("/auth/config", get(auth_config))
        .route("/auth/login", post(login))
        .route("/auth/logout", post(logout))
        .route("/auth/me", get(me))
        .route("/auth/saml/login", get(saml_login))
        .route("/auth/saml/callback", post(saml_callback))
        // Profile routes
        .route("/profile", get(get_profile))
        .route("/profile", patch(update_profile))
        .route("/profile/password", post(change_password))
        // User management routes
        .route("/users", get(list_users))
        .route("/users", post(create_user))
        .route("/users/{id}", get(get_user))
        .route("/users/{id}", patch(update_user))
        .route("/users/{id}", axum::routing::delete(delete_user))
        .route("/users/{id}/reset-password", post(reset_password))
        .with_state(state)
}
