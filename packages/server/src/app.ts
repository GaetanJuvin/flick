import Fastify, { type FastifyError } from 'fastify';
import cookie from '@fastify/cookie';
import cors from '@fastify/cors';
import formbody from '@fastify/formbody';
import { AppError } from './shared/errors.js';
import { loggerConfig } from './shared/logger.js';

export function buildApp() {
  const app = Fastify({ logger: loggerConfig });

  // Plugins
  app.register(cors, {
    origin: process.env.CORS_ORIGIN ?? 'http://localhost:4321',
    credentials: true,
  });

  app.register(cookie, {
    secret: process.env.SESSION_SECRET ?? 'dev-secret',
  });

  app.register(formbody);

  // Error handler
  app.setErrorHandler((err: FastifyError | AppError, request, reply) => {
    if (err instanceof AppError) {
      return reply.status(err.statusCode).send({
        error: { code: err.code, message: err.message },
      });
    }

    // Zod validation errors
    if (err.name === 'ZodError') {
      return reply.status(422).send({
        error: { code: 'VALIDATION_ERROR', message: err.message },
      });
    }

    // Fastify validation errors
    if ('validation' in err && err.validation) {
      return reply.status(422).send({
        error: { code: 'VALIDATION_ERROR', message: err.message },
      });
    }

    // Fastify built-in errors (JSON parse, content-type, etc.) — use their statusCode
    if ('statusCode' in err && typeof err.statusCode === 'number' && err.statusCode < 500) {
      return reply.status(err.statusCode).send({
        error: { code: err.code ?? 'BAD_REQUEST', message: err.message },
      });
    }

    request.log.error(err);
    return reply.status(500).send({
      error: { code: 'INTERNAL_ERROR', message: 'Internal server error' },
    });
  });

  // Health check
  app.get('/health', async () => ({ status: 'ok' }));

  return app;
}
