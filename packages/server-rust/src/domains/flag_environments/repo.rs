use chrono::{DateTime, Utc};
use serde::Serialize;
use serde_json::Value;
use sqlx::PgPool;
use uuid::Uuid;

#[derive(Debug, sqlx::FromRow, Serialize)]
pub struct FlagEnvironmentRow {
    pub id: Uuid,
    pub flag_id: Uuid,
    pub environment_id: Uuid,
    pub enabled: bool,
    pub gate_config: Value,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

pub async fn find_by_flag(
    db: &PgPool,
    flag_id: Uuid,
) -> Result<Vec<FlagEnvironmentRow>, sqlx::Error> {
    sqlx::query_as::<_, FlagEnvironmentRow>(
        "SELECT fe.*
         FROM flag_environments fe
         JOIN environments e ON e.id = fe.environment_id
         WHERE fe.flag_id = $1
         ORDER BY e.sort_order ASC",
    )
    .bind(flag_id)
    .fetch_all(db)
    .await
}

pub async fn find_by_id(
    db: &PgPool,
    id: Uuid,
) -> Result<Option<FlagEnvironmentRow>, sqlx::Error> {
    sqlx::query_as::<_, FlagEnvironmentRow>(
        "SELECT * FROM flag_environments WHERE id = $1",
    )
    .bind(id)
    .fetch_optional(db)
    .await
}

pub async fn find_by_flag_and_env(
    db: &PgPool,
    flag_id: Uuid,
    env_id: Uuid,
) -> Result<Option<FlagEnvironmentRow>, sqlx::Error> {
    sqlx::query_as::<_, FlagEnvironmentRow>(
        "SELECT * FROM flag_environments WHERE flag_id = $1 AND environment_id = $2",
    )
    .bind(flag_id)
    .bind(env_id)
    .fetch_optional(db)
    .await
}

pub async fn find_by_env(
    db: &PgPool,
    env_id: Uuid,
) -> Result<Vec<FlagEnvironmentRow>, sqlx::Error> {
    sqlx::query_as::<_, FlagEnvironmentRow>(
        "SELECT * FROM flag_environments WHERE environment_id = $1",
    )
    .bind(env_id)
    .fetch_all(db)
    .await
}

pub async fn update(
    db: &PgPool,
    id: Uuid,
    enabled: Option<bool>,
    gate_config: Option<Value>,
) -> Result<Option<FlagEnvironmentRow>, sqlx::Error> {
    if enabled.is_none() && gate_config.is_none() {
        return find_by_id(db, id).await;
    }

    let mut query = String::from("UPDATE flag_environments SET updated_at = NOW()");
    let mut param_idx = 0u32;

    if enabled.is_some() {
        param_idx += 1;
        query.push_str(&format!(", enabled = ${param_idx}"));
    }

    if gate_config.is_some() {
        param_idx += 1;
        query.push_str(&format!(", gate_config = ${param_idx}"));
    }

    param_idx += 1;
    query.push_str(&format!(" WHERE id = ${param_idx} RETURNING *"));

    let mut q = sqlx::query_as::<_, FlagEnvironmentRow>(&query);

    if let Some(e) = enabled {
        q = q.bind(e);
    }

    if let Some(gc) = gate_config {
        q = q.bind(gc);
    }

    q = q.bind(id);

    q.fetch_optional(db).await
}

pub async fn toggle(
    db: &PgPool,
    id: Uuid,
) -> Result<Option<FlagEnvironmentRow>, sqlx::Error> {
    sqlx::query_as::<_, FlagEnvironmentRow>(
        "UPDATE flag_environments SET enabled = NOT enabled, updated_at = NOW()
         WHERE id = $1 RETURNING *",
    )
    .bind(id)
    .fetch_optional(db)
    .await
}
