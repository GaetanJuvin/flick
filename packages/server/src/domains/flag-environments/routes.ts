import type { FastifyInstance } from 'fastify';
import { updateFlagEnvironmentSchema } from '@flick/shared';
import { authenticate } from '../../shared/auth.js';
import { requireAdmin } from '../../shared/rbac.js';
import * as service from './service.js';

export async function registerFlagEnvRoutes(app: FastifyInstance) {
  app.addHook('onRequest', authenticate);

  app.get('/projects/:projectId/flags/:flagId/environments', async (request) => {
    const { flagId } = request.params as { flagId: string };
    const envs = await service.getFlagEnvironments(flagId);
    return { data: envs };
  });

  app.get('/projects/:projectId/flags/:flagId/environments/:envId', async (request) => {
    const { flagId, envId } = request.params as { flagId: string; envId: string };
    const fe = await service.getFlagEnvironment(flagId, envId);
    return { data: fe };
  });

  app.patch('/projects/:projectId/flags/:flagId/environments/:envId', { preHandler: [requireAdmin] }, async (request) => {
    const { flagId, envId } = request.params as { flagId: string; envId: string };
    const input = updateFlagEnvironmentSchema.parse(request.body);
    const fe = await service.updateFlagEnvironment(flagId, envId, input);
    return { data: fe };
  });

  app.post('/projects/:projectId/flags/:flagId/environments/:envId/toggle', { preHandler: [requireAdmin] }, async (request) => {
    const { flagId, envId } = request.params as { flagId: string; envId: string };
    const fe = await service.toggleFlagEnvironment(flagId, envId);
    return { data: fe };
  });
}
