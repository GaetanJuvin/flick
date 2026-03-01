import type { FastifyInstance } from 'fastify';
import { createUserSchema, updateUserSchema, loginSchema, updateProfileSchema, changePasswordSchema, resetPasswordSchema } from '@flick/shared';
import { authenticate } from '../../shared/auth.js';
import { requireAdmin } from '../../shared/rbac.js';
import { authConfig } from '../../shared/config.js';
import { getSamlClient } from '../../shared/saml.js';
import { loginRateLimit, loginEmailRateLimit } from '../../shared/rate-limit.js';
import * as service from './service.js';

export async function registerUserRoutes(app: FastifyInstance) {
  // --- Auth config (public) ---
  app.get('/auth/config', async () => {
    const samlEnabled = authConfig.saml !== null && (authConfig.mode === 'saml' || authConfig.mode === 'both');
    return {
      data: {
        mode: authConfig.mode,
        saml_enabled: samlEnabled,
        saml_login_url: samlEnabled ? '/api/v1/auth/saml/login' : null,
      },
    };
  });

  // --- Password auth routes ---
  app.post('/auth/login', { preHandler: [loginRateLimit, loginEmailRateLimit] }, async (request, reply) => {
    const { email, password } = loginSchema.parse(request.body);
    const user = await service.login(email, password);
    const sessionData = Buffer.from(JSON.stringify(user)).toString('base64');
    reply.setCookie('session', sessionData, {
      httpOnly: true,
      secure: process.env.NODE_ENV === 'production',
      sameSite: 'lax',
      path: '/',
      maxAge: 60 * 60 * 24 * 7, // 7 days
    });
    return { data: user };
  });

  app.post('/auth/logout', async (_request, reply) => {
    reply.clearCookie('session', { path: '/' });
    return { data: { message: 'Logged out' } };
  });

  app.get('/auth/me', { preHandler: [authenticate] }, async (request) => {
    return { data: request.user };
  });

  // --- SAML auth routes ---
  app.get('/auth/saml/login', async (_request, reply) => {
    const saml = getSamlClient();
    if (!saml) {
      return reply.status(404).send({ error: { code: 'NOT_FOUND', message: 'SAML is not configured' } });
    }
    const url = await saml.generateLoginRequest();
    return reply.redirect(url);
  });

  app.post('/auth/saml/callback', async (request, reply) => {
    const saml = getSamlClient();
    if (!saml) {
      return reply.status(404).send({ error: { code: 'NOT_FOUND', message: 'SAML is not configured' } });
    }

    const profile = await saml.validateCallback(request.body as Record<string, string>);
    const user = await service.loginWithSaml(profile);

    const sessionData = Buffer.from(JSON.stringify(user)).toString('base64');
    reply.setCookie('session', sessionData, {
      httpOnly: true,
      secure: process.env.NODE_ENV === 'production',
      sameSite: 'lax',
      path: '/',
      maxAge: 60 * 60 * 24 * 7,
    });

    // Redirect to UI after successful SAML login
    const uiOrigin = process.env.CORS_ORIGIN ?? 'http://localhost:4321';
    return reply.redirect(uiOrigin);
  });

  // --- Profile routes (authenticated user) ---
  app.get('/profile', { preHandler: [authenticate] }, async (request) => {
    const user = await service.getProfile(request.user!.id);
    return { data: user };
  });

  app.patch('/profile', { preHandler: [authenticate] }, async (request, reply) => {
    const input = updateProfileSchema.parse(request.body);
    const user = await service.updateProfile(request.user!.id, input);

    // Re-issue session cookie with updated data
    const sessionData = Buffer.from(JSON.stringify(user)).toString('base64');
    reply.setCookie('session', sessionData, {
      httpOnly: true,
      secure: process.env.NODE_ENV === 'production',
      sameSite: 'lax',
      path: '/',
      maxAge: 60 * 60 * 24 * 7,
    });

    return { data: user };
  });

  app.post('/profile/password', { preHandler: [authenticate] }, async (request) => {
    const { current_password, new_password } = changePasswordSchema.parse(request.body);
    await service.changePassword(request.user!.id, current_password, new_password);
    return { data: { message: 'Password updated successfully' } };
  });

  // --- User management (requires auth + admin) ---
  app.get('/users', { preHandler: [authenticate, requireAdmin] }, async () => {
    const users = await service.listUsers();
    return { data: users };
  });

  app.post('/users', { preHandler: [authenticate, requireAdmin] }, async (request, reply) => {
    const input = createUserSchema.parse(request.body);
    const user = await service.createUser(input);
    return reply.status(201).send({ data: user });
  });

  app.get('/users/:id', { preHandler: [authenticate] }, async (request) => {
    const { id } = request.params as { id: string };
    const user = await service.getUser(id);
    return { data: user };
  });

  app.patch('/users/:id', { preHandler: [authenticate, requireAdmin] }, async (request) => {
    const { id } = request.params as { id: string };
    const input = updateUserSchema.parse(request.body);
    const user = await service.updateUser(id, input);
    return { data: user };
  });

  app.delete('/users/:id', { preHandler: [authenticate, requireAdmin] }, async (request, reply) => {
    const { id } = request.params as { id: string };
    await service.deleteUser(id);
    return reply.status(204).send();
  });

  // --- Admin: reset user password ---
  app.post('/users/:id/reset-password', { preHandler: [authenticate, requireAdmin] }, async (request) => {
    const { id } = request.params as { id: string };
    const { new_password } = resetPasswordSchema.parse(request.body);
    await service.adminResetPassword(request.user!.id, id, new_password);
    return { data: { message: 'Password reset successfully' } };
  });
}
