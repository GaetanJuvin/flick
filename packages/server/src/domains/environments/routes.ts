import type { FastifyInstance } from 'fastify';
import { createEnvironmentSchema, updateEnvironmentSchema } from '@flick/shared';
import { authenticate } from '../../shared/auth.js';
import { requireAdmin } from '../../shared/rbac.js';
import * as service from './service.js';

export async function registerEnvironmentRoutes(app: FastifyInstance) {
  app.addHook('onRequest', authenticate);

  app.get('/projects/:projectId/environments', async (request) => {
    const { projectId } = request.params as { projectId: string };
    const environments = await service.listEnvironments(projectId);
    return { data: environments };
  });

  app.post('/projects/:projectId/environments', { preHandler: [requireAdmin] }, async (request, reply) => {
    const { projectId } = request.params as { projectId: string };
    const input = createEnvironmentSchema.parse(request.body);
    const env = await service.createEnvironment(projectId, input);
    return reply.status(201).send({ data: env });
  });

  app.patch('/projects/:projectId/environments/:id', { preHandler: [requireAdmin] }, async (request) => {
    const { id } = request.params as { id: string };
    const input = updateEnvironmentSchema.parse(request.body);
    const env = await service.updateEnvironment(id, input);
    return { data: env };
  });

  app.delete('/projects/:projectId/environments/:id', { preHandler: [requireAdmin] }, async (request, reply) => {
    const { id } = request.params as { id: string };
    await service.deleteEnvironment(id);
    return reply.status(204).send();
  });
}
