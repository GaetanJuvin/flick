#!/usr/bin/env node
import { McpServer } from '@modelcontextprotocol/sdk/server/mcp.js';
import { StdioServerTransport } from '@modelcontextprotocol/sdk/server/stdio.js';
import { registerFlagTools } from './tools/flags.js';
import { registerEnvironmentTools } from './tools/environments.js';
import { registerGroupTools } from './tools/groups.js';
import { registerAuditTools } from './tools/audit.js';
import { registerFlagResources } from './resources/flags.js';

const server = new McpServer({
  name: 'flick',
  version: '0.1.0',
});

// Register tools
registerFlagTools(server);
registerEnvironmentTools(server);
registerGroupTools(server);
registerAuditTools(server);

// Register resources
registerFlagResources(server);

// Start server
const transport = new StdioServerTransport();
await server.connect(transport);
