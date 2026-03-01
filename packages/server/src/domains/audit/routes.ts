import type { FastifyInstance } from 'fastify';
import { authenticate } from '../../shared/auth.js';
import * as service from './service.js';

export async function registerAuditRoutes(app: FastifyInstance) {
  app.addHook('onRequest', authenticate);

  app.get('/projects/:projectId/audit', async (request) => {
    const { projectId } = request.params as { projectId: string };
    const query = request.query as {
      entity_type?: string;
      entity_id?: string;
      actor_id?: string;
      action?: string;
      cursor?: string;
      limit?: string;
    };

    const result = await service.getAuditLog(projectId, {
      entity_type: query.entity_type,
      entity_id: query.entity_id,
      actor_id: query.actor_id,
      action: query.action,
      cursor: query.cursor,
      limit: query.limit ? parseInt(query.limit, 10) : undefined,
    });

    return {
      data: result.entries,
      cursor: result.cursor,
      has_more: result.has_more,
    };
  });
}
