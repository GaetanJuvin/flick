use chrono::{DateTime, Utc};
use serde::Serialize;
use serde_json::json;
use sqlx::PgPool;
use uuid::Uuid;

use crate::error::AppError;

#[derive(Debug, sqlx::FromRow, Serialize)]
pub struct FlagRow {
    pub id: Uuid,
    pub project_id: Uuid,
    pub key: String,
    pub name: String,
    pub description: String,
    pub gate_type: String,
    pub tags: Vec<String>,
    pub archived: bool,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

pub async fn find_by_project(
    db: &PgPool,
    project_id: Uuid,
    archived: Option<bool>,
    tags: Option<Vec<String>>,
) -> Result<Vec<FlagRow>, sqlx::Error> {
    let mut query = String::from("SELECT * FROM flags WHERE project_id = $1");
    let mut param_idx = 1u32;

    if archived.is_some() {
        param_idx += 1;
        query.push_str(&format!(" AND archived = ${param_idx}"));
    }

    if tags.is_some() {
        param_idx += 1;
        query.push_str(&format!(" AND tags && ${param_idx}"));
    }

    query.push_str(" ORDER BY created_at DESC");

    let mut q = sqlx::query_as::<_, FlagRow>(&query).bind(project_id);

    if let Some(archived) = archived {
        q = q.bind(archived);
    }

    if let Some(tags) = tags {
        q = q.bind(tags);
    }

    q.fetch_all(db).await
}

pub async fn find_by_id(db: &PgPool, id: Uuid) -> Result<Option<FlagRow>, sqlx::Error> {
    sqlx::query_as::<_, FlagRow>("SELECT * FROM flags WHERE id = $1")
        .bind(id)
        .fetch_optional(db)
        .await
}

pub async fn find_by_key(
    db: &PgPool,
    project_id: Uuid,
    key: &str,
) -> Result<Option<FlagRow>, sqlx::Error> {
    sqlx::query_as::<_, FlagRow>(
        "SELECT * FROM flags WHERE project_id = $1 AND key = $2",
    )
    .bind(project_id)
    .bind(key)
    .fetch_optional(db)
    .await
}

pub async fn create(
    db: &PgPool,
    project_id: Uuid,
    key: &str,
    name: &str,
    description: &str,
    gate_type: &str,
    tags: &[String],
) -> Result<FlagRow, AppError> {
    let mut tx = db.begin().await.map_err(|e| AppError::Database(e))?;

    let flag = sqlx::query_as::<_, FlagRow>(
        "INSERT INTO flags (project_id, key, name, description, gate_type, tags)
         VALUES ($1, $2, $3, $4, $5, $6)
         RETURNING *",
    )
    .bind(project_id)
    .bind(key)
    .bind(name)
    .bind(description)
    .bind(gate_type)
    .bind(tags)
    .fetch_one(&mut *tx)
    .await
    .map_err(|e| AppError::Database(e))?;

    // Get all environment ids for this project
    let env_ids: Vec<(Uuid,)> =
        sqlx::query_as("SELECT id FROM environments WHERE project_id = $1")
            .bind(project_id)
            .fetch_all(&mut *tx)
            .await
            .map_err(|e| AppError::Database(e))?;

    // Default gate config based on gate type
    let gate_config = if gate_type == "percentage" {
        json!({"percentage": 0, "sticky": true})
    } else {
        json!({})
    };

    // Insert a flag_environment row for each environment
    for (env_id,) in &env_ids {
        sqlx::query(
            "INSERT INTO flag_environments (flag_id, environment_id, enabled, gate_config)
             VALUES ($1, $2, false, $3)",
        )
        .bind(flag.id)
        .bind(env_id)
        .bind(&gate_config)
        .execute(&mut *tx)
        .await
        .map_err(|e| AppError::Database(e))?;
    }

    tx.commit().await.map_err(|e| AppError::Database(e))?;

    Ok(flag)
}

pub async fn update(
    db: &PgPool,
    id: Uuid,
    name: Option<&str>,
    description: Option<&str>,
    tags: Option<&[String]>,
) -> Result<Option<FlagRow>, sqlx::Error> {
    if name.is_none() && description.is_none() && tags.is_none() {
        return find_by_id(db, id).await;
    }

    let mut query = String::from("UPDATE flags SET updated_at = NOW()");
    let mut param_idx = 0u32;

    if name.is_some() {
        param_idx += 1;
        query.push_str(&format!(", name = ${param_idx}"));
    }

    if description.is_some() {
        param_idx += 1;
        query.push_str(&format!(", description = ${param_idx}"));
    }

    if tags.is_some() {
        param_idx += 1;
        query.push_str(&format!(", tags = ${param_idx}"));
    }

    param_idx += 1;
    query.push_str(&format!(" WHERE id = ${param_idx} RETURNING *"));

    let mut q = sqlx::query_as::<_, FlagRow>(&query);

    if let Some(n) = name {
        q = q.bind(n);
    }

    if let Some(d) = description {
        q = q.bind(d);
    }

    if let Some(t) = tags {
        q = q.bind(t);
    }

    q = q.bind(id);

    q.fetch_optional(db).await
}

pub async fn archive(db: &PgPool, id: Uuid) -> Result<Option<FlagRow>, sqlx::Error> {
    sqlx::query_as::<_, FlagRow>(
        "UPDATE flags SET archived = true, updated_at = NOW() WHERE id = $1 RETURNING *",
    )
    .bind(id)
    .fetch_optional(db)
    .await
}

pub async fn restore(db: &PgPool, id: Uuid) -> Result<Option<FlagRow>, sqlx::Error> {
    sqlx::query_as::<_, FlagRow>(
        "UPDATE flags SET archived = false, updated_at = NOW() WHERE id = $1 RETURNING *",
    )
    .bind(id)
    .fetch_optional(db)
    .await
}

pub async fn remove(db: &PgPool, id: Uuid) -> Result<Option<Uuid>, sqlx::Error> {
    let row: Option<(Uuid,)> =
        sqlx::query_as("DELETE FROM flags WHERE id = $1 RETURNING id")
            .bind(id)
            .fetch_optional(db)
            .await?;
    Ok(row.map(|(id,)| id))
}
