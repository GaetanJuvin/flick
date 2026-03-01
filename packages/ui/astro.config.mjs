import { defineConfig } from 'astro/config';
import node from '@astrojs/node';
import react from '@astrojs/react';
import tailwind from '@astrojs/tailwind';

export default defineConfig({
  output: 'server',
  adapter: node({ mode: 'standalone' }),
  integrations: [react(), tailwind()],
  server: { port: parseInt(process.env.PORT || '4321'), host: process.env.HOST || '0.0.0.0' },
  security: { checkOrigin: false },
});
