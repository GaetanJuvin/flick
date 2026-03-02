use chrono::{DateTime, Utc};
use serde::Serialize;
use sqlx::PgPool;
use uuid::Uuid;

#[derive(Debug, sqlx::FromRow, Serialize)]
pub struct ProjectRow {
    pub id: Uuid,
    pub name: String,
    pub slug: String,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

pub async fn find_all(db: &PgPool) -> Result<Vec<ProjectRow>, sqlx::Error> {
    sqlx::query_as::<_, ProjectRow>("SELECT * FROM projects ORDER BY created_at DESC")
        .fetch_all(db)
        .await
}

pub async fn find_by_id(db: &PgPool, id: Uuid) -> Result<Option<ProjectRow>, sqlx::Error> {
    sqlx::query_as::<_, ProjectRow>("SELECT * FROM projects WHERE id = $1")
        .bind(id)
        .fetch_optional(db)
        .await
}

pub async fn find_by_slug(db: &PgPool, slug: &str) -> Result<Option<ProjectRow>, sqlx::Error> {
    sqlx::query_as::<_, ProjectRow>("SELECT * FROM projects WHERE slug = $1")
        .bind(slug)
        .fetch_optional(db)
        .await
}

pub async fn create(
    db: &PgPool,
    name: &str,
    slug: &str,
) -> Result<ProjectRow, sqlx::Error> {
    sqlx::query_as::<_, ProjectRow>(
        "INSERT INTO projects (name, slug) VALUES ($1, $2) RETURNING *",
    )
    .bind(name)
    .bind(slug)
    .fetch_one(db)
    .await
}

pub async fn update(
    db: &PgPool,
    id: Uuid,
    name: Option<&str>,
) -> Result<Option<ProjectRow>, sqlx::Error> {
    // If no fields to update, just return the existing row
    if name.is_none() {
        return find_by_id(db, id).await;
    }

    let mut query = String::from("UPDATE projects SET updated_at = NOW()");
    let mut param_idx = 1u32;

    if name.is_some() {
        param_idx += 1;
        query.push_str(&format!(", name = ${param_idx}"));
    }

    param_idx += 1;
    query.push_str(&format!(" WHERE id = ${param_idx} RETURNING *"));

    let mut q = sqlx::query_as::<_, ProjectRow>(&query);

    if let Some(n) = name {
        q = q.bind(n);
    }

    q = q.bind(id);

    q.fetch_optional(db).await
}
