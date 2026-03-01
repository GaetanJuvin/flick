import { z } from 'zod';
import type { McpServer } from '@modelcontextprotocol/sdk/server/mcp.js';
import { flickApi } from '../api-client.js';

export function registerFlagTools(server: McpServer) {
  server.tool(
    'flick_list_flags',
    'List all feature flags in a project',
    { project_id: z.string().describe('Project ID') },
    async ({ project_id }) => {
      const result = await flickApi<any>(`/projects/${project_id}/flags`);
      return { content: [{ type: 'text' as const, text: JSON.stringify(result.data, null, 2) }] };
    },
  );

  server.tool(
    'flick_get_flag',
    'Get details of a specific flag',
    { project_id: z.string(), flag_id: z.string() },
    async ({ project_id, flag_id }) => {
      const result = await flickApi<any>(`/projects/${project_id}/flags/${flag_id}`);
      return { content: [{ type: 'text' as const, text: JSON.stringify(result.data, null, 2) }] };
    },
  );

  server.tool(
    'flick_create_flag',
    'Create a new feature flag',
    {
      project_id: z.string(),
      key: z.string().describe('Flag key (kebab-case)'),
      name: z.string().describe('Display name'),
      gate_type: z.enum(['boolean', 'percentage', 'group']),
      description: z.string().optional(),
    },
    async ({ project_id, ...body }) => {
      const result = await flickApi<any>(`/projects/${project_id}/flags`, {
        method: 'POST',
        body: JSON.stringify(body),
      });
      return { content: [{ type: 'text' as const, text: JSON.stringify(result.data, null, 2) }] };
    },
  );

  server.tool(
    'flick_toggle_flag',
    'Toggle a flag on/off for a specific environment',
    { project_id: z.string(), flag_id: z.string(), env_id: z.string() },
    async ({ project_id, flag_id, env_id }) => {
      const result = await flickApi<any>(
        `/projects/${project_id}/flags/${flag_id}/environments/${env_id}/toggle`,
        { method: 'POST' },
      );
      return { content: [{ type: 'text' as const, text: `Flag toggled. Enabled: ${result.data.enabled}` }] };
    },
  );

  server.tool(
    'flick_update_flag_config',
    'Update gate config for a flag in a specific environment',
    {
      project_id: z.string(),
      flag_id: z.string(),
      env_id: z.string(),
      gate_config: z.record(z.unknown()).describe('Gate config (e.g. { percentage: 50, sticky: true })'),
    },
    async ({ project_id, flag_id, env_id, gate_config }) => {
      const result = await flickApi<any>(
        `/projects/${project_id}/flags/${flag_id}/environments/${env_id}`,
        { method: 'PATCH', body: JSON.stringify({ gate_config }) },
      );
      return { content: [{ type: 'text' as const, text: JSON.stringify(result.data, null, 2) }] };
    },
  );

  server.tool(
    'flick_archive_flag',
    'Archive a feature flag',
    { project_id: z.string(), flag_id: z.string() },
    async ({ project_id, flag_id }) => {
      const result = await flickApi<any>(
        `/projects/${project_id}/flags/${flag_id}/archive`,
        { method: 'POST' },
      );
      return { content: [{ type: 'text' as const, text: `Flag archived: ${result.data.key}` }] };
    },
  );

  server.tool(
    'flick_evaluate_flag',
    'Evaluate a flag for a given context',
    {
      env_id: z.string().describe('Environment ID'),
      flag_key: z.string(),
      context_key: z.string().describe('User/entity key'),
      attributes: z.record(z.unknown()).optional(),
    },
    async ({ env_id, flag_key, context_key, attributes }) => {
      const result = await flickApi<any>('/evaluate', {
        method: 'POST',
        headers: { 'X-Environment-Id': env_id } as Record<string, string>,
        body: JSON.stringify({
          flag_key,
          context: { key: context_key, attributes: attributes ?? {} },
        }),
      });
      return { content: [{ type: 'text' as const, text: JSON.stringify(result.data, null, 2) }] };
    },
  );
}
