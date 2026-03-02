use chrono::{DateTime, Utc};
use serde::Serialize;
use sqlx::{FromRow, PgPool};
use uuid::Uuid;

#[derive(Debug, FromRow, Serialize)]
pub struct GroupRow {
    pub id: Uuid,
    pub project_id: Uuid,
    pub name: String,
    pub slug: String,
    pub description: String,
    pub rules: serde_json::Value,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, FromRow, Serialize)]
pub struct FlagGroupRow {
    pub id: Uuid,
    pub flag_environment_id: Uuid,
    pub group_id: Uuid,
    pub created_at: DateTime<Utc>,
}

pub async fn find_by_project(db: &PgPool, project_id: Uuid) -> Result<Vec<GroupRow>, sqlx::Error> {
    sqlx::query_as::<_, GroupRow>(
        "SELECT * FROM groups WHERE project_id = $1 ORDER BY name ASC",
    )
    .bind(project_id)
    .fetch_all(db)
    .await
}

pub async fn find_by_id(db: &PgPool, id: Uuid) -> Result<Option<GroupRow>, sqlx::Error> {
    sqlx::query_as::<_, GroupRow>("SELECT * FROM groups WHERE id = $1")
        .bind(id)
        .fetch_optional(db)
        .await
}

pub async fn find_by_slug(
    db: &PgPool,
    project_id: Uuid,
    slug: &str,
) -> Result<Option<GroupRow>, sqlx::Error> {
    sqlx::query_as::<_, GroupRow>(
        "SELECT * FROM groups WHERE project_id = $1 AND slug = $2",
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
    description: &str,
    rules: serde_json::Value,
) -> Result<GroupRow, sqlx::Error> {
    sqlx::query_as::<_, GroupRow>(
        r#"
        INSERT INTO groups (id, project_id, name, slug, description, rules, created_at, updated_at)
        VALUES (gen_random_uuid(), $1, $2, $3, $4, $5, now(), now())
        RETURNING *
        "#,
    )
    .bind(project_id)
    .bind(name)
    .bind(slug)
    .bind(description)
    .bind(rules)
    .fetch_one(db)
    .await
}

pub async fn update(
    db: &PgPool,
    id: Uuid,
    name: Option<&str>,
    description: Option<&str>,
    rules: Option<serde_json::Value>,
) -> Result<Option<GroupRow>, sqlx::Error> {
    let mut query = String::from("UPDATE groups SET updated_at = now()");
    let mut param_index = 1u32;
    let mut binds: Vec<Box<dyn FnOnce(sqlx::query::QueryAs<'_, sqlx::Postgres, GroupRow, sqlx::postgres::PgArguments>) -> sqlx::query::QueryAs<'_, sqlx::Postgres, GroupRow, sqlx::postgres::PgArguments> + Send>> = Vec::new();

    if let Some(n) = name {
        param_index += 1;
        query.push_str(&format!(", name = ${}", param_index));
        let n = n.to_string();
        binds.push(Box::new(move |q| q.bind(n)));
    }

    if let Some(d) = description {
        param_index += 1;
        query.push_str(&format!(", description = ${}", param_index));
        let d = d.to_string();
        binds.push(Box::new(move |q| q.bind(d)));
    }

    if let Some(r) = rules {
        param_index += 1;
        query.push_str(&format!(", rules = ${}", param_index));
        binds.push(Box::new(move |q| q.bind(r)));
    }

    query.push_str(" WHERE id = $1 RETURNING *");

    let mut q = sqlx::query_as::<_, GroupRow>(&query).bind(id);
    for bind_fn in binds {
        q = bind_fn(q);
    }

    q.fetch_optional(db).await
}

pub async fn remove(db: &PgPool, id: Uuid) -> Result<bool, sqlx::Error> {
    let result = sqlx::query_scalar::<_, Uuid>("DELETE FROM groups WHERE id = $1 RETURNING id")
        .bind(id)
        .fetch_optional(db)
        .await?;
    Ok(result.is_some())
}

pub async fn find_groups_for_flag_env(
    db: &PgPool,
    flag_environment_id: Uuid,
) -> Result<Vec<GroupRow>, sqlx::Error> {
    sqlx::query_as::<_, GroupRow>(
        r#"
        SELECT g.*
        FROM groups g
        JOIN flag_groups fg ON fg.group_id = g.id
        WHERE fg.flag_environment_id = $1
        ORDER BY g.name ASC
        "#,
    )
    .bind(flag_environment_id)
    .fetch_all(db)
    .await
}

pub async fn add_group_to_flag(
    db: &PgPool,
    flag_environment_id: Uuid,
    group_id: Uuid,
) -> Result<FlagGroupRow, sqlx::Error> {
    let inserted = sqlx::query_as::<_, FlagGroupRow>(
        r#"
        INSERT INTO flag_groups (id, flag_environment_id, group_id, created_at)
        VALUES (gen_random_uuid(), $1, $2, now())
        ON CONFLICT DO NOTHING
        RETURNING *
        "#,
    )
    .bind(flag_environment_id)
    .bind(group_id)
    .fetch_optional(db)
    .await?;

    match inserted {
        Some(row) => Ok(row),
        None => {
            // Already exists, fetch the existing row
            sqlx::query_as::<_, FlagGroupRow>(
                "SELECT * FROM flag_groups WHERE flag_environment_id = $1 AND group_id = $2",
            )
            .bind(flag_environment_id)
            .bind(group_id)
            .fetch_one(db)
            .await
        }
    }
}

pub async fn remove_group_from_flag(
    db: &PgPool,
    flag_environment_id: Uuid,
    group_id: Uuid,
) -> Result<bool, sqlx::Error> {
    let result = sqlx::query(
        "DELETE FROM flag_groups WHERE flag_environment_id = $1 AND group_id = $2",
    )
    .bind(flag_environment_id)
    .bind(group_id)
    .execute(db)
    .await?;
    Ok(result.rows_affected() > 0)
}
