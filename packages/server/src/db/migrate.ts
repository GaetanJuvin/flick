import { readFileSync, readdirSync } from 'node:fs';
import { join, dirname, resolve } from 'node:path';
import { fileURLToPath } from 'node:url';
import dotenv from 'dotenv';
import pg from 'pg';

const __dirname = dirname(fileURLToPath(import.meta.url));
dotenv.config({ path: resolve(__dirname, '../../../../.env') });

const DATABASE_URL = process.env.DATABASE_URL ?? 'postgresql://flick:flick_dev@localhost:5432/flick';

async function migrate() {
  const pool = new pg.Pool({ connectionString: DATABASE_URL });

  try {
    // Create migrations tracking table
    await pool.query(`
      CREATE TABLE IF NOT EXISTS _migrations (
        id SERIAL PRIMARY KEY,
        name VARCHAR(255) NOT NULL UNIQUE,
        applied_at TIMESTAMPTZ NOT NULL DEFAULT now()
      )
    `);

    // Check which migrations have been applied
    const { rows: applied } = await pool.query('SELECT name FROM _migrations ORDER BY id');
    const appliedNames = new Set(applied.map((r: { name: string }) => r.name));

    // Scan migrations directory for .sql files
    const migrationsDir = join(__dirname, 'migrations');
    const files = readdirSync(migrationsDir)
      .filter((f) => f.endsWith('.sql'))
      .sort();

    for (const file of files) {
      if (appliedNames.has(file)) {
        console.log(`Skipping (already applied): ${file}`);
        continue;
      }

      console.log(`Applying migration: ${file}`);
      const sql = readFileSync(join(migrationsDir, file), 'utf-8');

      await pool.query('BEGIN');
      try {
        await pool.query(sql);
        await pool.query('INSERT INTO _migrations (name) VALUES ($1)', [file]);
        await pool.query('COMMIT');
        console.log(`  Applied: ${file}`);
      } catch (err) {
        await pool.query('ROLLBACK');
        throw err;
      }
    }

    console.log('Migrations complete.');
  } finally {
    await pool.end();
  }
}

migrate().catch((err) => {
  console.error('Migration failed:', err);
  process.exit(1);
});
