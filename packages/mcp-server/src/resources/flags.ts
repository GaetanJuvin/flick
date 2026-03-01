import type { McpServer } from '@modelcontextprotocol/sdk/server/mcp.js';
import { flickApi } from '../api-client.js';

export function registerFlagResources(server: McpServer) {
  server.resource(
    'flick-projects',
    'flick://projects',
    async (uri) => {
      const result = await flickApi<any>('/projects');
      return {
        contents: [{
          uri: uri.href,
          mimeType: 'application/json',
          text: JSON.stringify(result.data, null, 2),
        }],
      };
    },
  );

  server.resource(
    'flick-project-flags',
    'flick://projects/{slug}/flags',
    async (uri) => {
      const slug = uri.pathname.split('/')[2];
      // Look up project by slug
      const projects = await flickApi<any>('/projects');
      const project = projects.data.find((p: any) => p.slug === slug);
      if (!project) {
        return { contents: [{ uri: uri.href, mimeType: 'text/plain', text: 'Project not found' }] };
      }
      const result = await flickApi<any>(`/projects/${project.id}/flags`);
      return {
        contents: [{
          uri: uri.href,
          mimeType: 'application/json',
          text: JSON.stringify(result.data, null, 2),
        }],
      };
    },
  );

  server.resource(
    'flick-project-environments',
    'flick://projects/{slug}/environments',
    async (uri) => {
      const slug = uri.pathname.split('/')[2];
      const projects = await flickApi<any>('/projects');
      const project = projects.data.find((p: any) => p.slug === slug);
      if (!project) {
        return { contents: [{ uri: uri.href, mimeType: 'text/plain', text: 'Project not found' }] };
      }
      const result = await flickApi<any>(`/projects/${project.id}/environments`);
      return {
        contents: [{
          uri: uri.href,
          mimeType: 'application/json',
          text: JSON.stringify(result.data, null, 2),
        }],
      };
    },
  );

  server.resource(
    'flick-project-groups',
    'flick://projects/{slug}/groups',
    async (uri) => {
      const slug = uri.pathname.split('/')[2];
      const projects = await flickApi<any>('/projects');
      const project = projects.data.find((p: any) => p.slug === slug);
      if (!project) {
        return { contents: [{ uri: uri.href, mimeType: 'text/plain', text: 'Project not found' }] };
      }
      const result = await flickApi<any>(`/projects/${project.id}/groups`);
      return {
        contents: [{
          uri: uri.href,
          mimeType: 'application/json',
          text: JSON.stringify(result.data, null, 2),
        }],
      };
    },
  );

  server.resource(
    'flick-project-audit',
    'flick://projects/{slug}/audit',
    async (uri) => {
      const slug = uri.pathname.split('/')[2];
      const projects = await flickApi<any>('/projects');
      const project = projects.data.find((p: any) => p.slug === slug);
      if (!project) {
        return { contents: [{ uri: uri.href, mimeType: 'text/plain', text: 'Project not found' }] };
      }
      const result = await flickApi<any>(`/projects/${project.id}/audit?limit=20`);
      return {
        contents: [{
          uri: uri.href,
          mimeType: 'application/json',
          text: JSON.stringify(result.data, null, 2),
        }],
      };
    },
  );
}
