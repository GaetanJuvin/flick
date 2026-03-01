import { z } from 'zod';
import type { McpServer } from '@modelcontextprotocol/sdk/server/mcp.js';
import { flickApi } from '../api-client.js';

export function registerAuditTools(server: McpServer) {
  server.tool(
    'flick_get_audit_log',
    'Get audit log entries for a project',
    {
      project_id: z.string(),
      entity_type: z.string().optional(),
      action: z.string().optional(),
      limit: z.number().optional(),
    },
    async ({ project_id, entity_type, action, limit }) => {
      const params = new URLSearchParams();
      if (entity_type) params.set('entity_type', entity_type);
      if (action) params.set('action', action);
      if (limit) params.set('limit', String(limit));
      const qs = params.toString();
      const result = await flickApi<any>(`/projects/${project_id}/audit${qs ? `?${qs}` : ''}`);
      return { content: [{ type: 'text' as const, text: JSON.stringify(result.data, null, 2) }] };
    },
  );
}
