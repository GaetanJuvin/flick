import { z } from 'zod';
import type { McpServer } from '@modelcontextprotocol/sdk/server/mcp.js';
import { flickApi } from '../api-client.js';

export function registerEnvironmentTools(server: McpServer) {
  server.tool(
    'flick_list_environments',
    'List all environments in a project',
    { project_id: z.string() },
    async ({ project_id }) => {
      const result = await flickApi<any>(`/projects/${project_id}/environments`);
      return { content: [{ type: 'text' as const, text: JSON.stringify(result.data, null, 2) }] };
    },
  );
}
