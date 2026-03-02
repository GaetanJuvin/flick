use chrono::{DateTime, Utc};
use serde::Serialize;
use sqlx::{FromRow, PgPool};
use uuid::Uuid;

// ── Row types ──────────────────────────────────────────────────────────────

#[derive(Debug, FromRow, Serialize)]
pub struct WebhookRow {
    pub id: Uuid,
    pub project_id: Uuid,
    pub url: String,
    pub secret: String,
    pub events: Vec<String>,
    pub status: String,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, FromRow, Serialize)]
pub struct WebhookDeliveryRow {
    pub id: Uuid,
    pub webhook_id: Uuid,
    pub event: String,
    pub payload: serde_json::Value,
    pub response_status: Option<i32>,
    pub response_body: Option<String>,
    pub status: String,
    pub attempted_at: DateTime<Utc>,
}

// ── Queries ────────────────────────────────────────────────────────────────

pub async fn find_by_project(
    db: &PgPool,
    project_id: Uuid,
) -> Result<Vec<WebhookRow>, sqlx::Error> {
    sqlx::query_as::<_, WebhookRow>(
        "SELECT * FROM webhooks WHERE project_id = $1 ORDER BY created_at DESC",
    )
    .bind(project_id)
    .fetch_all(db)
    .await
}

pub async fn find_by_id(db: &PgPool, id: Uuid) -> Result<Option<WebhookRow>, sqlx::Error> {
    sqlx::query_as::<_, WebhookRow>("SELECT * FROM webhooks WHERE id = $1")
        .bind(id)
        .fetch_optional(db)
        .await
}

pub async fn find_by_project_and_event(
    db: &PgPool,
    project_id: Uuid,
    event: &str,
) -> Result<Vec<WebhookRow>, sqlx::Error> {
    sqlx::query_as::<_, WebhookRow>(
        "SELECT * FROM webhooks WHERE project_id = $1 AND status = 'active' AND $2 = ANY(events)",
    )
    .bind(project_id)
    .bind(event)
    .fetch_all(db)
    .await
}

pub async fn create(
    db: &PgPool,
    project_id: Uuid,
    url: &str,
    secret: &str,
    events: &[String],
) -> Result<WebhookRow, sqlx::Error> {
    sqlx::query_as::<_, WebhookRow>(
        r#"
        INSERT INTO webhooks (id, project_id, url, secret, events, status, created_at, updated_at)
        VALUES (gen_random_uuid(), $1, $2, $3, $4, 'active', now(), now())
        RETURNING *
        "#,
    )
    .bind(project_id)
    .bind(url)
    .bind(secret)
    .bind(events)
    .fetch_one(db)
    .await
}

pub async fn update(
    db: &PgPool,
    id: Uuid,
    url: Option<&str>,
    events: Option<&[String]>,
    status: Option<&str>,
) -> Result<Option<WebhookRow>, sqlx::Error> {
    let mut sets: Vec<String> = Vec::new();
    let mut param_index: u32 = 1;

    // We build the SET clause dynamically. Because sqlx doesn't support truly
    // dynamic bind lists in a straightforward way, we use a raw query approach
    // with format! for the SQL and bind parameters positionally.

    if url.is_some() {
        sets.push(format!("url = ${param_index}"));
        param_index += 1;
    }
    if events.is_some() {
        sets.push(format!("events = ${param_index}"));
        param_index += 1;
    }
    if status.is_some() {
        sets.push(format!("status = ${param_index}"));
        param_index += 1;
    }

    if sets.is_empty() {
        return find_by_id(db, id).await;
    }

    sets.push(format!("updated_at = now()"));

    let sql = format!(
        "UPDATE webhooks SET {} WHERE id = ${} RETURNING *",
        sets.join(", "),
        param_index
    );

    let mut query = sqlx::query_as::<_, WebhookRow>(&sql);

    if let Some(v) = url {
        query = query.bind(v);
    }
    if let Some(v) = events {
        query = query.bind(v);
    }
    if let Some(v) = status {
        query = query.bind(v);
    }
    query = query.bind(id);

    query.fetch_optional(db).await
}

pub async fn remove(db: &PgPool, id: Uuid) -> Result<bool, sqlx::Error> {
    let result = sqlx::query("DELETE FROM webhooks WHERE id = $1")
        .bind(id)
        .execute(db)
        .await?;
    Ok(result.rows_affected() > 0)
}

pub async fn create_delivery(
    db: &PgPool,
    webhook_id: Uuid,
    event: &str,
    payload: serde_json::Value,
    response_status: Option<i32>,
    response_body: Option<String>,
    status: &str,
) -> Result<WebhookDeliveryRow, sqlx::Error> {
    sqlx::query_as::<_, WebhookDeliveryRow>(
        r#"
        INSERT INTO webhook_deliveries (id, webhook_id, event, payload, response_status, response_body, status, attempted_at)
        VALUES (gen_random_uuid(), $1, $2, $3, $4, $5, $6, now())
        RETURNING *
        "#,
    )
    .bind(webhook_id)
    .bind(event)
    .bind(payload)
    .bind(response_status)
    .bind(response_body)
    .bind(status)
    .fetch_one(db)
    .await
}

pub async fn find_deliveries(
    db: &PgPool,
    webhook_id: Uuid,
    limit: i64,
) -> Result<Vec<WebhookDeliveryRow>, sqlx::Error> {
    sqlx::query_as::<_, WebhookDeliveryRow>(
        "SELECT * FROM webhook_deliveries WHERE webhook_id = $1 ORDER BY attempted_at DESC LIMIT $2",
    )
    .bind(webhook_id)
    .bind(limit)
    .fetch_all(db)
    .await
}
