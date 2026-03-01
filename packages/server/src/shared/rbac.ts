import type { FastifyRequest, FastifyReply } from 'fastify';
import { ForbiddenError } from './errors.js';
import type { UserRole } from '@flick/shared';

/**
 * RBAC middleware factory: requires the user to have one of the specified roles.
 */
export function requireRole(...roles: UserRole[]) {
  return async (request: FastifyRequest, _reply: FastifyReply): Promise<void> => {
    const user = request.user;
    if (!user) {
      throw new ForbiddenError('Authentication required');
    }
    if (!roles.includes(user.role)) {
      throw new ForbiddenError(`Requires role: ${roles.join(' or ')}`);
    }
  };
}

/**
 * Shortcut: require admin role.
 */
export const requireAdmin = requireRole('admin');
