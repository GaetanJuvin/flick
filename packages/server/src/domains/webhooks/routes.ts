import type { FastifyInstance } from 'fastify';
import { createWebhookSchema, updateWebhookSchema } from '@flick/shared';
import { authenticate } from '../../shared/auth.js';
import { requireAdmin } from '../../shared/rbac.js';
import * as service from './service.js';

export async function registerWebhookRoutes(app: FastifyInstance) {
  app.addHook('onRequest', authenticate);

  app.get('/projects/:projectId/webhooks', async (request) => {
    const { projectId } = request.params as { projectId: string };
    const webhooks = await service.listWebhooks(projectId);
    return { data: webhooks };
  });

  app.post('/projects/:projectId/webhooks', { preHandler: [requireAdmin] }, async (request, reply) => {
    const { projectId } = request.params as { projectId: string };
    const input = createWebhookSchema.parse(request.body);
    const webhook = await service.createWebhook(projectId, input);
    return reply.status(201).send({ data: webhook });
  });

  app.patch('/projects/:projectId/webhooks/:id', { preHandler: [requireAdmin] }, async (request) => {
    const { id } = request.params as { id: string };
    const input = updateWebhookSchema.parse(request.body);
    const webhook = await service.updateWebhook(id, input);
    return { data: webhook };
  });

  app.delete('/projects/:projectId/webhooks/:id', { preHandler: [requireAdmin] }, async (request, reply) => {
    const { id } = request.params as { id: string };
    await service.deleteWebhook(id);
    return reply.status(204).send();
  });

  app.post('/projects/:projectId/webhooks/:id/test', { preHandler: [requireAdmin] }, async (request) => {
    const { id } = request.params as { id: string };
    const delivery = await service.testWebhook(id);
    return { data: delivery };
  });

  app.get('/projects/:projectId/webhooks/:id/deliveries', async (request) => {
    const { id } = request.params as { id: string };
    const deliveries = await service.getDeliveries(id);
    return { data: deliveries };
  });
}
