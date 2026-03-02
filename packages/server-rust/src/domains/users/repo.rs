use chrono::{DateTime, Utc};
use serde::Serialize;
use sqlx::{FromRow, PgPool};
use uuid::Uuid;

const USER_COLUMNS: &str = "id, email, name, role, auth_method, created_at, updated_at";

// ── Row types ──────────────────────────────────────────────────────────────

#[derive(Debug, Clone, FromRow, Serialize)]
pub struct UserRow {
    pub id: Uuid,
    pub email: String,
    pub name: String,
    pub role: String,
    pub auth_method: String,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, FromRow)]
pub struct UserWithHashRow {
    pub id: Uuid,
    pub email: String,
    pub name: String,
    pub role: String,
    pub auth_method: String,
    pub password_hash: Option<String>,
    pub saml_name_id: Option<String>,
    pub saml_issuer: Option<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

// ── Queries ────────────────────────────────────────────────────────────────

pub async fn find_all(db: &PgPool) -> Result<Vec<UserRow>, sqlx::Error> {
    let sql = format!(
        "SELECT {} FROM users ORDER BY created_at DESC",
        USER_COLUMNS
    );
    sqlx::query_as::<_, UserRow>(&sql).fetch_all(db).await
}

pub async fn find_by_id(db: &PgPool, id: Uuid) -> Result<Option<UserRow>, sqlx::Error> {
    let sql = format!("SELECT {} FROM users WHERE id = $1", USER_COLUMNS);
    sqlx::query_as::<_, UserRow>(&sql)
        .bind(id)
        .fetch_optional(db)
        .await
}

pub async fn find_by_id_with_hash(
    db: &PgPool,
    id: Uuid,
) -> Result<Option<UserWithHashRow>, sqlx::Error> {
    sqlx::query_as::<_, UserWithHashRow>("SELECT * FROM users WHERE id = $1")
        .bind(id)
        .fetch_optional(db)
        .await
}

pub async fn find_by_email(
    db: &PgPool,
    email: &str,
) -> Result<Option<UserWithHashRow>, sqlx::Error> {
    sqlx::query_as::<_, UserWithHashRow>("SELECT * FROM users WHERE email = $1")
        .bind(email)
        .fetch_optional(db)
        .await
}

pub async fn find_by_saml_identity(
    db: &PgPool,
    name_id: &str,
    issuer: &str,
) -> Result<Option<UserWithHashRow>, sqlx::Error> {
    sqlx::query_as::<_, UserWithHashRow>(
        "SELECT * FROM users WHERE saml_name_id = $1 AND saml_issuer = $2",
    )
    .bind(name_id)
    .bind(issuer)
    .fetch_optional(db)
    .await
}

pub async fn create(
    db: &PgPool,
    email: &str,
    name: &str,
    password_hash: &str,
    role: &str,
) -> Result<UserRow, sqlx::Error> {
    let sql = format!(
        r#"
        INSERT INTO users (id, email, name, password_hash, role, auth_method, created_at, updated_at)
        VALUES (gen_random_uuid(), $1, $2, $3, $4, 'password', now(), now())
        RETURNING {}
        "#,
        USER_COLUMNS
    );
    sqlx::query_as::<_, UserRow>(&sql)
        .bind(email)
        .bind(name)
        .bind(password_hash)
        .bind(role)
        .fetch_one(db)
        .await
}

pub async fn create_saml_user(
    db: &PgPool,
    email: &str,
    name: &str,
    saml_name_id: &str,
    saml_issuer: &str,
) -> Result<UserRow, sqlx::Error> {
    let sql = format!(
        r#"
        INSERT INTO users (id, email, name, role, auth_method, saml_name_id, saml_issuer, created_at, updated_at)
        VALUES (gen_random_uuid(), $1, $2, 'viewer', 'saml', $3, $4, now(), now())
        RETURNING {}
        "#,
        USER_COLUMNS
    );
    sqlx::query_as::<_, UserRow>(&sql)
        .bind(email)
        .bind(name)
        .bind(saml_name_id)
        .bind(saml_issuer)
        .fetch_one(db)
        .await
}

pub async fn update(
    db: &PgPool,
    id: Uuid,
    name: Option<&str>,
    email: Option<&str>,
    role: Option<&str>,
) -> Result<Option<UserRow>, sqlx::Error> {
    let mut sets: Vec<String> = Vec::new();
    let mut param_index: u32 = 1;

    if name.is_some() {
        sets.push(format!("name = ${param_index}"));
        param_index += 1;
    }
    if email.is_some() {
        sets.push(format!("email = ${param_index}"));
        param_index += 1;
    }
    if role.is_some() {
        sets.push(format!("role = ${param_index}"));
        param_index += 1;
    }

    if sets.is_empty() {
        return find_by_id(db, id).await;
    }

    sets.push("updated_at = now()".to_string());

    let sql = format!(
        "UPDATE users SET {} WHERE id = ${} RETURNING {}",
        sets.join(", "),
        param_index,
        USER_COLUMNS
    );

    let mut query = sqlx::query_as::<_, UserRow>(&sql);

    if let Some(v) = name {
        query = query.bind(v);
    }
    if let Some(v) = email {
        query = query.bind(v);
    }
    if let Some(v) = role {
        query = query.bind(v);
    }
    query = query.bind(id);

    query.fetch_optional(db).await
}

pub async fn update_profile(
    db: &PgPool,
    id: Uuid,
    name: Option<&str>,
    email: Option<&str>,
) -> Result<Option<UserRow>, sqlx::Error> {
    let mut sets: Vec<String> = Vec::new();
    let mut param_index: u32 = 1;

    if name.is_some() {
        sets.push(format!("name = ${param_index}"));
        param_index += 1;
    }
    if email.is_some() {
        sets.push(format!("email = ${param_index}"));
        param_index += 1;
    }

    if sets.is_empty() {
        return find_by_id(db, id).await;
    }

    sets.push("updated_at = now()".to_string());

    let sql = format!(
        "UPDATE users SET {} WHERE id = ${} RETURNING {}",
        sets.join(", "),
        param_index,
        USER_COLUMNS
    );

    let mut query = sqlx::query_as::<_, UserRow>(&sql);

    if let Some(v) = name {
        query = query.bind(v);
    }
    if let Some(v) = email {
        query = query.bind(v);
    }
    query = query.bind(id);

    query.fetch_optional(db).await
}

pub async fn update_password_hash(
    db: &PgPool,
    id: Uuid,
    password_hash: &str,
) -> Result<(), sqlx::Error> {
    sqlx::query("UPDATE users SET password_hash = $1, updated_at = now() WHERE id = $2")
        .bind(password_hash)
        .bind(id)
        .execute(db)
        .await?;
    Ok(())
}

pub async fn remove(db: &PgPool, id: Uuid) -> Result<bool, sqlx::Error> {
    let result = sqlx::query("DELETE FROM users WHERE id = $1")
        .bind(id)
        .execute(db)
        .await?;
    Ok(result.rows_affected() > 0)
}
