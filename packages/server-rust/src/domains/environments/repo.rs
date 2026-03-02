use chrono::{DateTime, Utc};
use serde::Serialize;
use sqlx::PgPool;
use uuid::Uuid;

#[derive(Debug, sqlx::FromRow, Serialize)]
pub struct EnvironmentRow {
    pub id: Uuid,
    pub project_id: Uuid,
    pub name: String,
    pub slug: String,
    pub color: String,
    pub sort_order: i32,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

pub async fn find_by_project(
    db: &PgPool,
    project_id: Uuid,
) -> Result<Vec<EnvironmentRow>, sqlx::Error> {
    sqlx::query_as::<_, EnvironmentRow>(
        "SELECT * FROM environments WHERE project_id = $1 ORDER BY sort_order ASC",
    )
    .bind(project_id)
    .fetch_all(db)
    .await
}

pub async fn find_by_id(db: &PgPool, id: Uuid) -> Result<Option<EnvironmentRow>, sqlx::Error> {
    sqlx::query_as::<_, EnvironmentRow>("SELECT * FROM environments WHERE id = $1")
        .bind(id)
        .fetch_optional(db)
        .await
}

pub async fn find_by_slug(
    db: &PgPool,
    project_id: Uuid,
    slug: &str,
) -> Result<Option<EnvironmentRow>, sqlx::Error> {
    sqlx::query_as::<_, EnvironmentRow>(
        "SELECT * FROM environments WHERE project_id = $1 AND slug = $2",
    )
    .bind(project_id)
    .bind(slug)
    .fetch_optional(db)
    .await
}

pub async fn create(
    db: &PgPool,
    project_id: Uuid,
    name: &str,
    slug: &str,
    color: &str,
    sort_order: i32,
) -> Result<EnvironmentRow, sqlx::Error> {
    sqlx::query_as::<_, EnvironmentRow>(
        "INSERT INTO environments (project_id, name, slug, color, sort_order) \
         VALUES ($1, $2, $3, $4, $5) RETURNING *",
    )
    .bind(project_id)
    .bind(name)
    .bind(slug)
    .bind(color)
    .bind(sort_order)
    .fetch_one(db)
    .await
}

pub async fn update(
    db: &PgPool,
    id: Uuid,
    name: Option<&str>,
    color: Option<&str>,
    sort_order: Option<i32>,
) -> Result<Option<EnvironmentRow>, sqlx::Error> {
    // If no fields to update, just return the existing row
    if name.is_none() && color.is_none() && sort_order.is_none() {
        return find_by_id(db, id).await;
    }

    let mut set_clauses = vec!["updated_at = NOW()".to_string()];
    let mut param_count = 0u32;

    if name.is_some() {
        param_count += 1;
        set_clauses.push(format!("name = ${param_count}"));
    }
    if color.is_some() {
        param_count += 1;
        set_clauses.push(format!("color = ${param_count}"));
    }
    if sort_order.is_some() {
        param_count += 1;
        set_clauses.push(format!("sort_order = ${param_count}"));
    }

    param_count += 1;
    let query = format!(
        "UPDATE environments SET {} WHERE id = ${param_count} RETURNING *",
        set_clauses.join(", ")
    );

    let mut q = sqlx::query_as::<_, EnvironmentRow>(&query);

    if let Some(n) = name {
        q = q.bind(n);
    }
    if let Some(c) = color {
        q = q.bind(c);
    }
    if let Some(s) = sort_order {
        q = q.bind(s);
    }

    q = q.bind(id);

    q.fetch_optional(db).await
}

pub async fn remove(db: &PgPool, id: Uuid) -> Result<Option<Uuid>, sqlx::Error> {
    let row: Option<(Uuid,)> =
        sqlx::query_as("DELETE FROM environments WHERE id = $1 RETURNING id")
            .bind(id)
            .fetch_optional(db)
            .await?;

    Ok(row.map(|(rid,)| rid))
}
