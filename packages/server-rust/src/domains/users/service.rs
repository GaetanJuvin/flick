use serde::Serialize;
use sqlx::PgPool;
use uuid::Uuid;

use crate::config::{AppConfig, AuthMode};
use crate::error::AppError;

use super::repo;

// ── Types ──────────────────────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize)]
pub struct SessionUser {
    pub id: Uuid,
    pub email: String,
    pub name: String,
    pub role: String,
    pub auth_method: String,
}

impl From<&repo::UserWithHashRow> for SessionUser {
    fn from(u: &repo::UserWithHashRow) -> Self {
        Self {
            id: u.id,
            email: u.email.clone(),
            name: u.name.clone(),
            role: u.role.clone(),
            auth_method: u.auth_method.clone(),
        }
    }
}

impl From<&repo::UserRow> for SessionUser {
    fn from(u: &repo::UserRow) -> Self {
        Self {
            id: u.id,
            email: u.email.clone(),
            name: u.name.clone(),
            role: u.role.clone(),
            auth_method: u.auth_method.clone(),
        }
    }
}

pub struct CreateUserInput {
    pub email: String,
    pub name: String,
    pub password: String,
    pub role: String,
}

pub struct UpdateUserInput {
    pub name: Option<String>,
    pub email: Option<String>,
    pub role: Option<String>,
}

pub struct UpdateProfileInput {
    pub name: Option<String>,
    pub email: Option<String>,
}

pub struct SamlProfile {
    pub name_id: String,
    pub issuer: String,
    pub email: String,
    pub name: String,
}

// ── Service functions ──────────────────────────────────────────────────────

pub async fn list_users(db: &PgPool) -> Result<Vec<repo::UserRow>, AppError> {
    repo::find_all(db).await.map_err(AppError::from)
}

pub async fn get_user(db: &PgPool, id: Uuid) -> Result<repo::UserRow, AppError> {
    repo::find_by_id(db, id)
        .await?
        .ok_or_else(|| AppError::not_found("User", Some(&id.to_string())))
}

pub async fn get_profile(db: &PgPool, id: Uuid) -> Result<repo::UserRow, AppError> {
    repo::find_by_id(db, id)
        .await?
        .ok_or_else(|| AppError::not_found("User", Some(&id.to_string())))
}

pub async fn create_user(
    db: &PgPool,
    config: &AppConfig,
    input: CreateUserInput,
) -> Result<repo::UserRow, AppError> {
    if config.auth_mode == AuthMode::Saml {
        return Err(AppError::validation(
            "Cannot create password users when auth mode is SAML",
        ));
    }

    // Check email conflict
    if repo::find_by_email(db, &input.email).await?.is_some() {
        return Err(AppError::conflict("A user with this email already exists"));
    }

    let password_hash =
        bcrypt::hash(&input.password, 12).map_err(|e| AppError::Internal(anyhow::anyhow!(e.to_string())))?;

    repo::create(db, &input.email, &input.name, &password_hash, &input.role)
        .await
        .map_err(AppError::from)
}

pub async fn update_user(
    db: &PgPool,
    id: Uuid,
    input: UpdateUserInput,
) -> Result<repo::UserRow, AppError> {
    repo::update(
        db,
        id,
        input.name.as_deref(),
        input.email.as_deref(),
        input.role.as_deref(),
    )
    .await?
    .ok_or_else(|| AppError::not_found("User", Some(&id.to_string())))
}

pub async fn update_profile(
    db: &PgPool,
    id: Uuid,
    input: UpdateProfileInput,
) -> Result<repo::UserRow, AppError> {
    // Check email conflict with a different user
    if let Some(ref email) = input.email {
        if let Some(existing) = repo::find_by_email(db, email).await? {
            if existing.id != id {
                return Err(AppError::conflict("A user with this email already exists"));
            }
        }
    }

    repo::update_profile(db, id, input.name.as_deref(), input.email.as_deref())
        .await?
        .ok_or_else(|| AppError::not_found("User", Some(&id.to_string())))
}

pub async fn change_password(
    db: &PgPool,
    id: Uuid,
    current_password: &str,
    new_password: &str,
) -> Result<(), AppError> {
    let user = repo::find_by_id_with_hash(db, id)
        .await?
        .ok_or_else(|| AppError::not_found("User", Some(&id.to_string())))?;

    if user.auth_method != "password" {
        return Err(AppError::validation(
            "Password change is only available for password-based accounts",
        ));
    }

    let hash = user
        .password_hash
        .ok_or_else(|| AppError::Internal(anyhow::anyhow!("No password hash found")))?;

    let valid =
        bcrypt::verify(current_password, &hash).map_err(|e| AppError::Internal(anyhow::anyhow!(e.to_string())))?;

    if !valid {
        return Err(AppError::unauthorized("Current password is incorrect"));
    }

    let new_hash =
        bcrypt::hash(new_password, 12).map_err(|e| AppError::Internal(anyhow::anyhow!(e.to_string())))?;

    repo::update_password_hash(db, id, &new_hash).await?;

    Ok(())
}

pub async fn admin_reset_password(
    db: &PgPool,
    target_id: Uuid,
    new_password: &str,
) -> Result<(), AppError> {
    let user = repo::find_by_id_with_hash(db, target_id)
        .await?
        .ok_or_else(|| AppError::not_found("User", Some(&target_id.to_string())))?;

    if user.auth_method != "password" {
        return Err(AppError::validation(
            "Password reset is only available for password-based accounts",
        ));
    }

    let new_hash =
        bcrypt::hash(new_password, 12).map_err(|e| AppError::Internal(anyhow::anyhow!(e.to_string())))?;

    repo::update_password_hash(db, target_id, &new_hash).await?;

    Ok(())
}

pub async fn delete_user(db: &PgPool, id: Uuid) -> Result<(), AppError> {
    let deleted = repo::remove(db, id).await?;
    if !deleted {
        return Err(AppError::not_found("User", Some(&id.to_string())));
    }
    Ok(())
}

pub async fn login(
    db: &PgPool,
    config: &AppConfig,
    email: &str,
    password: &str,
) -> Result<SessionUser, AppError> {
    if config.auth_mode == AuthMode::Saml {
        return Err(AppError::validation(
            "Password login is disabled when auth mode is SAML",
        ));
    }

    let user = repo::find_by_email(db, email)
        .await?
        .ok_or_else(|| AppError::unauthorized("Invalid email or password"))?;

    if user.auth_method != "password" {
        return Err(AppError::unauthorized("Invalid email or password"));
    }

    let hash = user
        .password_hash
        .as_deref()
        .ok_or_else(|| AppError::unauthorized("Invalid email or password"))?;

    let valid = bcrypt::verify(password, hash).map_err(|e| AppError::Internal(anyhow::anyhow!(e.to_string())))?;

    if !valid {
        return Err(AppError::unauthorized("Invalid email or password"));
    }

    Ok(SessionUser::from(&user))
}

pub async fn login_with_saml(db: &PgPool, profile: SamlProfile) -> Result<SessionUser, AppError> {
    // Try to find by SAML identity first
    if let Some(user) =
        repo::find_by_saml_identity(db, &profile.name_id, &profile.issuer).await?
    {
        return Ok(SessionUser::from(&user));
    }

    // Check if a user with this email already exists (collision)
    if let Some(existing) = repo::find_by_email(db, &profile.email).await? {
        // Email collision with a non-SAML user
        if existing.auth_method != "saml" {
            return Err(AppError::conflict(
                "A user with this email already exists with a different auth method",
            ));
        }
        // SAML user exists with same email but different name_id/issuer — treat as conflict
        return Err(AppError::conflict(
            "A SAML user with this email already exists with a different identity",
        ));
    }

    // JIT provision: create new SAML user
    let user =
        repo::create_saml_user(db, &profile.email, &profile.name, &profile.name_id, &profile.issuer)
            .await?;

    Ok(SessionUser::from(&user))
}
