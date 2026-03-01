import type { FastifyInstance } from 'fastify';
import { createFlagSchema, updateFlagSchema } from '@flick/shared';
import { authenticate } from '../../shared/auth.js';
import { requireAdmin } from '../../shared/rbac.js';
import * as service from './service.js';

export async function registerFlagRoutes(app: FastifyInstance) {
  app.addHook('onRequest', authenticate);

  app.get('/projects/:projectId/flags', async (request) => {
    const { projectId } = request.params as { projectId: string };
    const query = request.query as { archived?: string; tags?: string };
    const opts: { archived?: boolean; tags?: string[] } = {};
    if (query.archived !== undefined) opts.archived = query.archived === 'true';
    if (query.tags) opts.tags = query.tags.split(',');
    const flags = await service.listFlags(projectId, opts);
    return { data: flags };
  });

  app.post('/projects/:projectId/flags', { preHandler: [requireAdmin] }, async (request, reply) => {
    const { projectId } = request.params as { projectId: string };
    const input = createFlagSchema.parse(request.body);
    const flag = await service.createFlag(projectId, input);
    return reply.status(201).send({ data: flag });
  });

  app.get('/projects/:projectId/flags/:id', async (request) => {
    const { id } = request.params as { id: string };
    const flag = await service.getFlag(id);
    return { data: flag };
  });

  app.patch('/projects/:projectId/flags/:id', { preHandler: [requireAdmin] }, async (request) => {
    const { id } = request.params as { id: string };
    const input = updateFlagSchema.parse(request.body);
    const flag = await service.updateFlag(id, input);
    return { data: flag };
  });

  app.delete('/projects/:projectId/flags/:id', { preHandler: [requireAdmin] }, async (request, reply) => {
    const { id } = request.params as { id: string };
    await service.deleteFlag(id);
    return reply.status(204).send();
  });

  app.post('/projects/:projectId/flags/:id/archive', { preHandler: [requireAdmin] }, async (request) => {
    const { id } = request.params as { id: string };
    const flag = await service.archiveFlag(id);
    return { data: flag };
  });

  app.post('/projects/:projectId/flags/:id/restore', { preHandler: [requireAdmin] }, async (request) => {
    const { id } = request.params as { id: string };
    const flag = await service.restoreFlag(id);
    return { data: flag };
  });
}
