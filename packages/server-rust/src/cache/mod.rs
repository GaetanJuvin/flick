pub mod keys;

use fred::prelude::*;
use serde::{de::DeserializeOwned, Serialize};
use tracing::warn;

pub const CACHE_TTL_FLAGS: i64 = 60;
pub const CACHE_TTL_FLAG: i64 = 60;
pub const CACHE_TTL_API_KEY: i64 = 300;

pub async fn connect(redis_url: &str) -> Result<fred::clients::RedisPool, RedisError> {
    let config = RedisConfig::from_url(redis_url)?;
    let pool = fred::clients::RedisPool::new(config, None, None, None, 4_usize)?;
    let _handle = pool.init().await?;
    Ok(pool)
}

pub async fn cache_get<T: DeserializeOwned>(redis: &fred::clients::RedisPool, key: &str) -> Option<T> {
    let raw: Option<String> = redis.get(key).await.ok()?;
    let raw = raw?;
    serde_json::from_str(&raw).ok()
}

pub async fn cache_set<T: Serialize>(
    redis: &fred::clients::RedisPool,
    key: &str,
    value: &T,
    ttl_seconds: i64,
) {
    let json = match serde_json::to_string(value) {
        Ok(j) => j,
        Err(e) => {
            warn!(error = %e, key = %key, "Failed to serialize cache value");
            return;
        }
    };
    let expiration = Some(Expiration::EX(ttl_seconds));
    if let Err(e) = redis
        .set::<(), _, _>(key, json, expiration, None, false)
        .await
    {
        warn!(error = %e, key = %key, "Failed to set cache");
    }
}

pub async fn cache_del(redis: &fred::clients::RedisPool, keys: &[&str]) {
    if keys.is_empty() {
        return;
    }
    let keys_vec: Vec<String> = keys.iter().map(|k| k.to_string()).collect();
    if let Err(e) = redis.del::<(), _>(keys_vec).await {
        warn!(error = %e, "Failed to delete cache keys");
    }
}

pub async fn invalidate_env_flags(redis: &fred::clients::RedisPool, env_id: &str) {
    use futures::StreamExt;
    use std::pin::pin;

    // Delete the environment flags list cache
    cache_del(redis, &[&keys::env_flags_key(env_id)]).await;

    // Also delete individual flag caches for this env using SCAN
    let pattern = format!("flick:env:{}:flag:*", env_id);
    let client = redis.next();
    let mut stream = pin!(client.scan_buffered(pattern, None, None));
    let mut to_delete: Vec<String> = Vec::new();
    while let Some(result) = stream.next().await {
        match result {
            Ok(key) => {
                if let Some(s) = key.into_string() {
                    to_delete.push(s);
                }
            }
            Err(e) => {
                warn!(error = %e, "Failed to scan keys for cache invalidation");
                break;
            }
        }
    }
    if !to_delete.is_empty() {
        let key_refs: Vec<&str> = to_delete.iter().map(|s| s.as_str()).collect();
        cache_del(redis, &key_refs).await;
    }
}
