import { z } from 'zod';
import type { McpServer } from '@modelcontextprotocol/sdk/server/mcp.js';
import { flickApi } from '../api-client.js';

export function registerGroupTools(server: McpServer) {
  server.tool(
    'flick_list_groups',
    'List all groups in a project',
    { project_id: z.string() },
    async ({ project_id }) => {
      const result = await flickApi<any>(`/projects/${project_id}/groups`);
      return { content: [{ type: 'text' as const, text: JSON.stringify(result.data, null, 2) }] };
    },
  );

  server.tool(
    'flick_create_group',
    'Create a new group with rules',
    {
      project_id: z.string(),
      name: z.string(),
      slug: z.string(),
      description: z.string().optional(),
      rules: z.array(z.object({
        attribute: z.string(),
        operator: z.string(),
        value: z.union([z.string(), z.array(z.string()), z.number()]),
      })),
    },
    async ({ project_id, ...body }) => {
      const result = await flickApi<any>(`/projects/${project_id}/groups`, {
        method: 'POST',
        body: JSON.stringify(body),
      });
      return { content: [{ type: 'text' as const, text: JSON.stringify(result.data, null, 2) }] };
    },
  );

  server.tool(
    'flick_add_group_to_flag',
    'Add a group to a flag in a specific environment',
    { project_id: z.string(), flag_id: z.string(), env_id: z.string(), group_id: z.string() },
    async ({ project_id, flag_id, env_id, group_id }) => {
      const result = await flickApi<any>(
        `/projects/${project_id}/flags/${flag_id}/environments/${env_id}/groups`,
        { method: 'POST', body: JSON.stringify({ group_id }) },
      );
      return { content: [{ type: 'text' as const, text: 'Group added to flag.' }] };
    },
  );

  server.tool(
    'flick_remove_group_from_flag',
    'Remove a group from a flag in a specific environment',
    { project_id: z.string(), flag_id: z.string(), env_id: z.string(), group_id: z.string() },
    async ({ project_id, flag_id, env_id, group_id }) => {
      await flickApi<any>(
        `/projects/${project_id}/flags/${flag_id}/environments/${env_id}/groups/${group_id}`,
        { method: 'DELETE' },
      );
      return { content: [{ type: 'text' as const, text: 'Group removed from flag.' }] };
    },
  );
}
