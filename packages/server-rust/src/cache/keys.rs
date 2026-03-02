/// Cache key for all flags in an environment
pub fn env_flags_key(env_id: &str) -> String {
    format!("flick:env:{}:flags", env_id)
}

/// Cache key for a single flag in an environment
pub fn env_flag_key(env_id: &str, flag_key: &str) -> String {
    format!("flick:env:{}:flag:{}", env_id, flag_key)
}

/// Cache key for an API key hash lookup
pub fn api_key_hash_key(hash: &str) -> String {
    format!("flick:apikey:{}", hash)
}

/// Rate limit key
pub fn rate_limit_key(prefix: &str, id: &str) -> String {
    format!("flick:rl:{}:{}", prefix, id)
}
