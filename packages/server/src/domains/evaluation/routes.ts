import type { FastifyInstance } from 'fastify';
import { evaluateRequestSchema, batchEvaluateRequestSchema } from '@flick/shared';
import { authenticateSdk } from '../../shared/auth.js';
import { evaluateRateLimit } from '../../shared/rate-limit.js';
import * as service from './service.js';

export async function registerEvaluationRoutes(app: FastifyInstance) {
  // SDK endpoints use API key auth + rate limiting
  app.addHook('onRequest', authenticateSdk);
  app.addHook('onRequest', evaluateRateLimit);

  app.post('/evaluate', async (request) => {
    const { flag_key, context } = evaluateRequestSchema.parse(request.body);
    const envId = await resolveEnvId(request);
    const result = await service.evaluate(envId, flag_key, context);
    return { data: result };
  });

  app.post('/evaluate/batch', async (request) => {
    const { context } = batchEvaluateRequestSchema.parse(request.body);
    const envId = await resolveEnvId(request);
    const results = await service.evaluateBatch(envId, context);
    return { data: { flags: results } };
  });

  app.get('/evaluate/config', async (request, reply) => {
    const envId = await resolveEnvId(request);
    const config = await service.getFullConfig(envId);

    // ETag support for SDK polling
    const etag = `"${config.version}"`;
    const ifNoneMatch = request.headers['if-none-match'];
    if (ifNoneMatch === etag) {
      return reply.status(304).send();
    }

    reply.header('ETag', etag);
    return { data: config };
  });
}

/**
 * Resolve the environment ID from the API key's environment binding
 * or from the X-Environment-Id header.
 */
async function resolveEnvId(request: any): Promise<string> {
  // API keys can be bound to an environment
  if (request.apiKeyEnvId) return request.apiKeyEnvId;

  // Or specified via header
  const envId = request.headers['x-environment-id'];
  if (envId) return envId as string;

  // Fallback: use query param
  const query = request.query as { environment_id?: string };
  if (query.environment_id) return query.environment_id;

  throw new Error('Environment ID required. Set X-Environment-Id header or bind API key to an environment.');
}
