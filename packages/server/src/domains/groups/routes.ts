import type { FastifyInstance } from 'fastify';
import { createGroupSchema, updateGroupSchema } from '@flick/shared';
import { authenticate } from '../../shared/auth.js';
import { requireAdmin } from '../../shared/rbac.js';
import * as service from './service.js';

export async function registerGroupRoutes(app: FastifyInstance) {
  app.addHook('onRequest', authenticate);

  app.get('/projects/:projectId/groups', async (request) => {
    const { projectId } = request.params as { projectId: string };
    const groups = await service.listGroups(projectId);
    return { data: groups };
  });

  app.post('/projects/:projectId/groups', { preHandler: [requireAdmin] }, async (request, reply) => {
    const { projectId } = request.params as { projectId: string };
    const input = createGroupSchema.parse(request.body);
    const group = await service.createGroup(projectId, input);
    return reply.status(201).send({ data: group });
  });

  app.get('/projects/:projectId/groups/:id', async (request) => {
    const { id } = request.params as { id: string };
    const group = await service.getGroup(id);
    return { data: group };
  });

  app.patch('/projects/:projectId/groups/:id', { preHandler: [requireAdmin] }, async (request) => {
    const { id } = request.params as { id: string };
    const input = updateGroupSchema.parse(request.body);
    const group = await service.updateGroup(id, input);
    return { data: group };
  });

  app.delete('/projects/:projectId/groups/:id', { preHandler: [requireAdmin] }, async (request, reply) => {
    const { id } = request.params as { id: string };
    await service.deleteGroup(id);
    return reply.status(204).send();
  });

  // Flag-group associations
  app.get('/projects/:projectId/flags/:flagId/environments/:envId/groups', async (request) => {
    const { flagId, envId } = request.params as { flagId: string; envId: string };
    const groups = await service.getGroupsForFlagEnv(flagId, envId);
    return { data: groups };
  });

  app.post('/projects/:projectId/flags/:flagId/environments/:envId/groups', { preHandler: [requireAdmin] }, async (request, reply) => {
    const { flagId, envId } = request.params as { flagId: string; envId: string };
    const { group_id } = request.body as { group_id: string };
    const result = await service.addGroupToFlag(flagId, envId, group_id);
    return reply.status(201).send({ data: result });
  });

  app.delete('/projects/:projectId/flags/:flagId/environments/:envId/groups/:groupId', { preHandler: [requireAdmin] }, async (request, reply) => {
    const { flagId, envId, groupId } = request.params as { flagId: string; envId: string; groupId: string };
    await service.removeGroupFromFlag(flagId, envId, groupId);
    return reply.status(204).send();
  });
}
