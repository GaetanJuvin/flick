import type { FastifyInstance } from 'fastify';
import { createApiKeySchema } from '@flick/shared';
import { authenticate } from '../../shared/auth.js';
import { requireAdmin } from '../../shared/rbac.js';
import * as service from './service.js';

export async function registerApiKeyRoutes(app: FastifyInstance) {
  app.addHook('onRequest', authenticate);
  app.addHook('preHandler', requireAdmin);

  app.get('/projects/:projectId/api-keys', async (request) => {
    const { projectId } = request.params as { projectId: string };
    const keys = await service.listApiKeys(projectId);
    return { data: keys };
  });

  app.post('/projects/:projectId/api-keys', async (request, reply) => {
    const { projectId } = request.params as { projectId: string };
    const input = createApiKeySchema.parse(request.body);
    const key = await service.createApiKey(projectId, input, request.user!);
    return reply.status(201).send({ data: key });
  });

  app.delete('/projects/:projectId/api-keys/:id', async (request, reply) => {
    const { id } = request.params as { id: string };
    await service.revokeApiKey(id);
    return reply.status(204).send();
  });
}
