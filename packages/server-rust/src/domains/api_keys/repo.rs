use chrono::{DateTime, Utc};
use serde::Serialize;
use sqlx::{FromRow, PgPool};
use uuid::Uuid;

#[derive(Debug, FromRow, Serialize)]
pub struct ApiKeyRow {
    pub id: Uuid,
    pub project_id: Uuid,
    pub name: String,
    pub key_prefix: String,
    pub key_hash: String,
    #[sqlx(rename = "type")]
    #[serde(rename = "type")]
    pub key_type: String,
    pub environment_id: Option<Uuid>,
    pub created_by: Uuid,
    pub last_used_at: Option<DateTime<Utc>>,
    pub created_at: DateTime<Utc>,
}

pub async fn find_by_project(
    db: &PgPool,
    project_id: Uuid,
) -> Result<Vec<ApiKeyRow>, sqlx::Error> {
    sqlx::query_as::<_, ApiKeyRow>(
        "SELECT * FROM api_keys WHERE project_id = $1 ORDER BY created_at DESC",
    )
    .bind(project_id)
    .fetch_all(db)
    .await
}

pub async fn find_by_id(db: &PgPool, id: Uuid) -> Result<Option<ApiKeyRow>, sqlx::Error> {
    sqlx::query_as::<_, ApiKeyRow>("SELECT * FROM api_keys WHERE id = $1")
        .bind(id)
        .fetch_optional(db)
        .await
}

pub async fn create(
    db: &PgPool,
    project_id: Uuid,
    name: &str,
    key_prefix: &str,
    key_hash: &str,
    key_type: &str,
    environment_id: Option<Uuid>,
    created_by: Uuid,
) -> Result<ApiKeyRow, sqlx::Error> {
    sqlx::query_as::<_, ApiKeyRow>(
        r#"
        INSERT INTO api_keys (id, project_id, name, key_prefix, key_hash, type, environment_id, created_by, created_at)
        VALUES (gen_random_uuid(), $1, $2, $3, $4, $5, $6, $7, now())
        RETURNING *
        "#,
    )
    .bind(project_id)
    .bind(name)
    .bind(key_prefix)
    .bind(key_hash)
    .bind(key_type)
    .bind(environment_id)
    .bind(created_by)
    .fetch_one(db)
    .await
}

pub async fn remove(db: &PgPool, id: Uuid) -> Result<Option<ApiKeyRow>, sqlx::Error> {
    sqlx::query_as::<_, ApiKeyRow>("DELETE FROM api_keys WHERE id = $1 RETURNING *")
        .bind(id)
        .fetch_optional(db)
        .await
}
