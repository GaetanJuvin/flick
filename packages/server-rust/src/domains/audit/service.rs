use serde_json::Value;
use sqlx::PgPool;
use uuid::Uuid;

use crate::auth::AuthUser;
use crate::error::AppError;

use super::repo::{self, AuditEntryRow};

pub async fn log(
    db: &PgPool,
    project_id: Uuid,
    actor: &AuthUser,
    action: &str,
    entity_type: &str,
    entity_id: &str,
    entity_name: &str,
    before_state: Option<Value>,
    after_state: Option<Value>,
) -> Result<AuditEntryRow, AppError> {
    let actor_id: Uuid = actor.id.parse().map_err(|_| AppError::validation("Invalid actor ID"))?;
    repo::create(
        db,
        project_id,
        actor_id,
        &actor.email,
        action,
        entity_type,
        entity_id,
        entity_name,
        before_state,
        after_state,
    )
    .await
    .map_err(AppError::from)
}

pub async fn get_audit_log(
    db: &PgPool,
    project_id: Uuid,
    entity_type: Option<&str>,
    entity_id: Option<&str>,
    actor_id: Option<Uuid>,
    action: Option<&str>,
    cursor: Option<&str>,
    limit: Option<i64>,
) -> Result<(Vec<AuditEntryRow>, Option<String>, bool), AppError> {
    repo::find_by_project(
        db,
        project_id,
        entity_type,
        entity_id,
        actor_id,
        action,
        cursor,
        limit,
    )
    .await
    .map_err(AppError::from)
}
