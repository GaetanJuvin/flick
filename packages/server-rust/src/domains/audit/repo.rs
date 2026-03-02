use base64::Engine;
use chrono::{DateTime, Utc};
use serde::Serialize;
use serde_json::Value;
use sqlx::{FromRow, PgPool};
use uuid::Uuid;

#[derive(Debug, FromRow, Serialize)]
pub struct AuditEntryRow {
    pub id: Uuid,
    pub project_id: Uuid,
    pub actor_id: Uuid,
    pub actor_email: String,
    pub action: String,
    pub entity_type: String,
    pub entity_id: String,
    pub entity_name: String,
    pub before_state: Option<Value>,
    pub after_state: Option<Value>,
    pub created_at: DateTime<Utc>,
}

pub async fn create(
    db: &PgPool,
    project_id: Uuid,
    actor_id: Uuid,
    actor_email: &str,
    action: &str,
    entity_type: &str,
    entity_id: &str,
    entity_name: &str,
    before_state: Option<Value>,
    after_state: Option<Value>,
) -> Result<AuditEntryRow, sqlx::Error> {
    sqlx::query_as::<_, AuditEntryRow>(
        r#"
        INSERT INTO audit_log (id, project_id, actor_id, actor_email, action, entity_type, entity_id, entity_name, before_state, after_state, created_at)
        VALUES (gen_random_uuid(), $1, $2, $3, $4, $5, $6, $7, $8, $9, now())
        RETURNING *
        "#,
    )
    .bind(project_id)
    .bind(actor_id)
    .bind(actor_email)
    .bind(action)
    .bind(entity_type)
    .bind(entity_id)
    .bind(entity_name)
    .bind(before_state)
    .bind(after_state)
    .fetch_one(db)
    .await
}

pub async fn find_by_project(
    db: &PgPool,
    project_id: Uuid,
    entity_type: Option<&str>,
    entity_id: Option<&str>,
    actor_id: Option<Uuid>,
    action: Option<&str>,
    cursor: Option<&str>,
    limit: Option<i64>,
) -> Result<(Vec<AuditEntryRow>, Option<String>, bool), sqlx::Error> {
    let limit = limit.unwrap_or(50).min(100);
    let fetch_limit = limit + 1;

    let mut query = String::from("SELECT * FROM audit_log WHERE project_id = $1");
    let mut param_index = 1u32;

    // We'll build this dynamically. Since sqlx doesn't support truly dynamic
    // queries with arbitrary bind types easily, we use a manual approach.
    let mut conditions = Vec::new();
    let mut entity_type_val = None;
    let mut entity_id_val = None;
    let mut actor_id_val = None;
    let mut action_val = None;
    let mut cursor_ts: Option<DateTime<Utc>> = None;

    if let Some(et) = entity_type {
        param_index += 1;
        conditions.push(format!("entity_type = ${}", param_index));
        entity_type_val = Some(et.to_string());
    }

    if let Some(eid) = entity_id {
        param_index += 1;
        conditions.push(format!("entity_id = ${}", param_index));
        entity_id_val = Some(eid.to_string());
    }

    if let Some(aid) = actor_id {
        param_index += 1;
        conditions.push(format!("actor_id = ${}", param_index));
        actor_id_val = Some(aid);
    }

    if let Some(a) = action {
        param_index += 1;
        conditions.push(format!("action = ${}", param_index));
        action_val = Some(a.to_string());
    }

    if let Some(c) = cursor {
        if let Ok(decoded) = base64::engine::general_purpose::STANDARD.decode(c) {
            if let Ok(ts_str) = String::from_utf8(decoded) {
                if let Ok(ts) = ts_str.parse::<DateTime<Utc>>() {
                    param_index += 1;
                    conditions.push(format!("created_at < ${}", param_index));
                    cursor_ts = Some(ts);
                }
            }
        }
    }

    for condition in &conditions {
        query.push_str(&format!(" AND {}", condition));
    }

    param_index += 1;
    query.push_str(&format!(
        " ORDER BY created_at DESC LIMIT ${}",
        param_index
    ));

    // Build and execute the query
    let mut q = sqlx::query_as::<_, AuditEntryRow>(&query).bind(project_id);

    if let Some(ref et) = entity_type_val {
        q = q.bind(et);
    }
    if let Some(ref eid) = entity_id_val {
        q = q.bind(eid);
    }
    if let Some(aid) = actor_id_val {
        q = q.bind(aid);
    }
    if let Some(ref a) = action_val {
        q = q.bind(a);
    }
    if let Some(ts) = cursor_ts {
        q = q.bind(ts);
    }

    q = q.bind(fetch_limit);

    let mut rows = q.fetch_all(db).await?;

    let has_more = rows.len() as i64 > limit;
    if has_more {
        rows.pop();
    }

    let next_cursor = if has_more {
        rows.last().map(|row| {
            base64::engine::general_purpose::STANDARD.encode(row.created_at.to_rfc3339())
        })
    } else {
        None
    };

    Ok((rows, next_cursor, has_more))
}
