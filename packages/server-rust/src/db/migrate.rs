use sqlx::PgPool;
use std::path::PathBuf;
use tracing::info;

/// Custom migration runner that uses the same `_migrations` table and .sql files
/// as the TypeScript server, allowing both servers to share the same database.
pub async fn run(pool: &PgPool) {
    // Ensure _migrations table exists
    sqlx::query(
        "CREATE TABLE IF NOT EXISTS _migrations (
            id SERIAL PRIMARY KEY,
            name VARCHAR(255) NOT NULL UNIQUE,
            executed_at TIMESTAMPTZ NOT NULL DEFAULT now()
        )"
    )
    .execute(pool)
    .await
    .expect("Failed to create _migrations table");

    // Find migration files
    let migrations_dir = find_migrations_dir();
    let mut entries: Vec<_> = std::fs::read_dir(&migrations_dir)
        .unwrap_or_else(|e| panic!("Failed to read migrations dir {:?}: {}", migrations_dir, e))
        .filter_map(|entry| {
            let entry = entry.ok()?;
            let name = entry.file_name().to_string_lossy().to_string();
            if name.ends_with(".sql") { Some((name, entry.path())) } else { None }
        })
        .collect();
    entries.sort_by(|a, b| a.0.cmp(&b.0));

    // Get already-executed migrations
    let executed: Vec<String> = sqlx::query_scalar("SELECT name FROM _migrations ORDER BY name")
        .fetch_all(pool)
        .await
        .expect("Failed to query _migrations");

    let executed_set: std::collections::HashSet<&str> = executed.iter().map(|s| s.as_str()).collect();

    for (name, path) in &entries {
        if executed_set.contains(name.as_str()) {
            continue;
        }

        info!(migration = %name, "Running migration");
        let sql = std::fs::read_to_string(path)
            .unwrap_or_else(|e| panic!("Failed to read migration {:?}: {}", path, e));

        // Execute the migration in a transaction
        let mut tx = pool.begin().await.expect("Failed to begin transaction");
        sqlx::query(&sql)
            .execute(&mut *tx)
            .await
            .unwrap_or_else(|e| panic!("Migration {} failed: {}", name, e));

        sqlx::query("INSERT INTO _migrations (name) VALUES ($1)")
            .bind(name)
            .execute(&mut *tx)
            .await
            .unwrap_or_else(|e| panic!("Failed to record migration {}: {}", name, e));

        tx.commit().await.expect("Failed to commit migration");
        info!(migration = %name, "Migration complete");
    }
}

fn find_migrations_dir() -> PathBuf {
    // Try symlink in our package first
    let local = PathBuf::from("migrations");
    if local.exists() {
        return local;
    }

    // Try relative to packages/server-rust/
    let relative = PathBuf::from("../server/src/db/migrations");
    if relative.exists() {
        return relative;
    }

    // Try from workspace root
    let workspace = PathBuf::from("packages/server/src/db/migrations");
    if workspace.exists() {
        return workspace;
    }

    panic!("Could not find migrations directory. Create a symlink: ln -s ../server/src/db/migrations migrations");
}
