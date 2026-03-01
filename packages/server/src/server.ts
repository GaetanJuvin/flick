import dotenv from 'dotenv';
import { resolve, dirname } from 'node:path';
import { fileURLToPath } from 'node:url';

const __dirname = dirname(fileURLToPath(import.meta.url));
dotenv.config({ path: resolve(__dirname, '../../../.env') });

// Dynamic imports — env vars must be loaded before these modules initialize
const { buildApp } = await import('./app.js');
const { redis } = await import('./shared/redis.js');
const { pool } = await import('./shared/db.js');

const PORT = parseInt(process.env.PORT ?? '3000', 10);
const HOST = process.env.HOST ?? '0.0.0.0';

async function start() {
  const app = buildApp();

  // Register domain routes
  const { registerProjectRoutes } = await import('./domains/projects/routes.js');
  const { registerEnvironmentRoutes } = await import('./domains/environments/routes.js');
  const { registerFlagRoutes } = await import('./domains/flags/routes.js');
  const { registerFlagEnvRoutes } = await import('./domains/flag-environments/routes.js');
  const { registerGroupRoutes } = await import('./domains/groups/routes.js');
  const { registerEvaluationRoutes } = await import('./domains/evaluation/routes.js');
  const { registerAuditRoutes } = await import('./domains/audit/routes.js');
  const { registerUserRoutes } = await import('./domains/users/routes.js');
  const { registerApiKeyRoutes } = await import('./domains/api-keys/routes.js');
  const { registerWebhookRoutes } = await import('./domains/webhooks/routes.js');

  app.register(registerProjectRoutes, { prefix: '/api/v1' });
  app.register(registerEnvironmentRoutes, { prefix: '/api/v1' });
  app.register(registerFlagRoutes, { prefix: '/api/v1' });
  app.register(registerFlagEnvRoutes, { prefix: '/api/v1' });
  app.register(registerGroupRoutes, { prefix: '/api/v1' });
  app.register(registerEvaluationRoutes, { prefix: '/api/v1' });
  app.register(registerAuditRoutes, { prefix: '/api/v1' });
  app.register(registerUserRoutes, { prefix: '/api/v1' });
  app.register(registerApiKeyRoutes, { prefix: '/api/v1' });
  app.register(registerWebhookRoutes, { prefix: '/api/v1' });

  // Connect Redis
  await redis.connect();

  await app.listen({ port: PORT, host: HOST });
  console.log(`Flick server listening on ${HOST}:${PORT}`);
}

start().catch((err) => {
  console.error('Failed to start server:', err);
  process.exit(1);
});

// Graceful shutdown
async function shutdown() {
  console.log('Shutting down...');
  await redis.quit();
  await pool.end();
  process.exit(0);
}

process.on('SIGTERM', shutdown);
process.on('SIGINT', shutdown);
