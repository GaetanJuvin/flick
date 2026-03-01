import type { FastifyInstance } from 'fastify';
import { createProjectSchema, updateProjectSchema } from '@flick/shared';
import { authenticate } from '../../shared/auth.js';
import { requireAdmin } from '../../shared/rbac.js';
import * as service from './service.js';

export async function registerProjectRoutes(app: FastifyInstance) {
  app.addHook('onRequest', authenticate);

  app.get('/projects', async () => {
    const projects = await service.listProjects();
    return { data: projects };
  });

  app.post('/projects', { preHandler: [requireAdmin] }, async (request, reply) => {
    const input = createProjectSchema.parse(request.body);
    const project = await service.createProject(input);
    return reply.status(201).send({ data: project });
  });

  app.get('/projects/:id', async (request) => {
    const { id } = request.params as { id: string };
    const project = await service.getProject(id);
    return { data: project };
  });

  app.patch('/projects/:id', { preHandler: [requireAdmin] }, async (request) => {
    const { id } = request.params as { id: string };
    const input = updateProjectSchema.parse(request.body);
    const project = await service.updateProject(id, input);
    return { data: project };
  });
}
