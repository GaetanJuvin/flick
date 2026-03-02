use std::collections::HashMap;

use sqlx::PgPool;
use tracing::warn;

use crate::cache::{self, keys, CACHE_TTL_FLAGS};
use crate::evaluation::{
    evaluate_flag, EvaluationContext, EvaluationResult, FlagConfig, FlagGroupConfig, FullFlagConfig,
    GroupRule,
};

// ---------------------------------------------------------------------------
// SQL row types
// ---------------------------------------------------------------------------

#[derive(sqlx::FromRow)]
struct FlagRow {
    key: String,
    gate_type: String,
    enabled: bool,
    gate_config: serde_json::Value,
    #[allow(dead_code)]
    archived: bool,
    flag_environment_id: uuid::Uuid,
}

#[derive(sqlx::FromRow)]
struct GroupRow {
    flag_environment_id: uuid::Uuid,
    id: uuid::Uuid,
    rules: serde_json::Value,
}

#[derive(sqlx::FromRow)]
struct EnvSlugRow {
    slug: String,
}

// ---------------------------------------------------------------------------
// Load flag configs (cache-first, then Postgres)
// ---------------------------------------------------------------------------

pub async fn load_flag_configs(
    db: &PgPool,
    redis: &fred::clients::RedisPool,
    env_id: &str,
) -> Result<Vec<FlagConfig>, sqlx::Error> {
    // Try Redis cache first
    let cache_key = keys::env_flags_key(env_id);
    if let Some(cached) = cache::cache_get::<Vec<FlagConfig>>(redis, &cache_key).await {
        return Ok(cached);
    }

    // Cache miss — query Postgres
    let flags = sqlx::query_as::<_, FlagRow>(
        "SELECT f.key, f.gate_type, fe.enabled, fe.gate_config, f.archived, \
                fe.id as flag_environment_id \
         FROM flags f \
         JOIN flag_environments fe ON fe.flag_id = f.id \
         WHERE fe.environment_id = $1 AND f.archived = false",
    )
    .bind(uuid::Uuid::parse_str(env_id).unwrap_or_default())
    .fetch_all(db)
    .await?;

    // Fetch groups for all flag_environments in this env
    let fe_ids: Vec<uuid::Uuid> = flags.iter().map(|f| f.flag_environment_id).collect();

    let groups: Vec<GroupRow> = if fe_ids.is_empty() {
        Vec::new()
    } else {
        sqlx::query_as::<_, GroupRow>(
            "SELECT fg.flag_environment_id, g.id, g.rules \
             FROM flag_groups fg \
             JOIN groups g ON g.id = fg.group_id \
             WHERE fg.flag_environment_id = ANY($1)",
        )
        .bind(&fe_ids)
        .fetch_all(db)
        .await?
    };

    // Index groups by flag_environment_id
    let mut groups_by_fe_id: HashMap<uuid::Uuid, Vec<&GroupRow>> = HashMap::new();
    for g in &groups {
        groups_by_fe_id
            .entry(g.flag_environment_id)
            .or_default()
            .push(g);
    }

    // Build FlagConfig vec
    let configs: Vec<FlagConfig> = flags
        .iter()
        .map(|f| {
            let flag_groups = groups_by_fe_id
                .get(&f.flag_environment_id)
                .map(|gs| {
                    gs.iter()
                        .map(|g| {
                            let rules: Vec<GroupRule> =
                                serde_json::from_value(g.rules.clone()).unwrap_or_else(|e| {
                                    warn!(
                                        error = %e,
                                        group_id = %g.id,
                                        "Failed to deserialize group rules"
                                    );
                                    Vec::new()
                                });
                            FlagGroupConfig {
                                id: g.id.to_string(),
                                rules,
                            }
                        })
                        .collect()
                })
                .unwrap_or_default();

            FlagConfig {
                key: f.key.clone(),
                gate_type: f.gate_type.clone(),
                enabled: f.enabled,
                gate_config: f.gate_config.clone(),
                groups: flag_groups,
            }
        })
        .collect();

    // Cache the result
    cache::cache_set(redis, &cache_key, &configs, CACHE_TTL_FLAGS).await;

    Ok(configs)
}

// ---------------------------------------------------------------------------
// Evaluate a single flag
// ---------------------------------------------------------------------------

pub async fn evaluate(
    db: &PgPool,
    redis: &fred::clients::RedisPool,
    env_id: &str,
    flag_key: &str,
    context: &EvaluationContext,
) -> Result<EvaluationResult, sqlx::Error> {
    let configs = load_flag_configs(db, redis, env_id).await?;
    let flag_config = configs.iter().find(|f| f.key == flag_key);
    let mut result = evaluate_flag(flag_config, context);

    // When flag was not found, set the requested key on the result
    if flag_config.is_none() {
        result.flag_key = flag_key.to_string();
    }

    Ok(result)
}

// ---------------------------------------------------------------------------
// Evaluate all flags (batch)
// ---------------------------------------------------------------------------

pub async fn evaluate_batch(
    db: &PgPool,
    redis: &fred::clients::RedisPool,
    env_id: &str,
    context: &EvaluationContext,
) -> Result<HashMap<String, EvaluationResult>, sqlx::Error> {
    let configs = load_flag_configs(db, redis, env_id).await?;
    let mut results = HashMap::with_capacity(configs.len());

    for config in &configs {
        let result = evaluate_flag(Some(config), context);
        results.insert(config.key.clone(), result);
    }

    Ok(results)
}

// ---------------------------------------------------------------------------
// Full config (for SDK polling with ETag)
// ---------------------------------------------------------------------------

pub async fn get_full_config(
    db: &PgPool,
    redis: &fred::clients::RedisPool,
    env_id: &str,
) -> Result<FullFlagConfig, sqlx::Error> {
    let configs = load_flag_configs(db, redis, env_id).await?;

    // Resolve environment slug
    let env_slug = sqlx::query_as::<_, EnvSlugRow>(
        "SELECT slug FROM environments WHERE id = $1",
    )
    .bind(uuid::Uuid::parse_str(env_id).unwrap_or_default())
    .fetch_optional(db)
    .await?;

    let environment = env_slug
        .map(|e| e.slug)
        .unwrap_or_else(|| env_id.to_string());

    // Generate MD5 version hash for ETag support
    let json_bytes = serde_json::to_string(&configs).unwrap_or_default();
    use md5::{Digest, Md5};
    let mut hasher = Md5::new();
    hasher.update(json_bytes.as_bytes());
    let version = format!("{:x}", hasher.finalize());

    Ok(FullFlagConfig {
        environment,
        flags: configs,
        version,
    })
}
