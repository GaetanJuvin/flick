import type { FastifyServerOptions } from 'fastify';

export const loggerConfig: FastifyServerOptions['logger'] = {
  level: process.env.LOG_LEVEL ?? 'info',
  transport: process.env.NODE_ENV === 'development'
    ? { target: 'pino-pretty', options: { colorize: true } }
    : undefined,
};
