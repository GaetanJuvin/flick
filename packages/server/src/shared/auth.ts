import type { FastifyRequest, FastifyReply } from 'fastify';
import { createHash } from 'node:crypto';
import { getOne } from './db.js';
import { cacheGet, cacheSet, apiKeyHashKey, CACHE_TTL } from './cache.js';
import { UnauthorizedError } from './errors.js';
import type { SessionUser } from '@flick/shared';

declare module 'fastify' {
  interface FastifyRequest {
    user?: SessionUser;
    projectId?: string;
    apiKeyType?: 'sdk' | 'management';
  }
}

function hashApiKey(raw: string): string {
  return createHash('sha256').update(raw).digest('hex');
}

interface CachedApiKey {
  id: string;
  project_id: string;
  type: 'sdk' | 'management';
  user: SessionUser;
}

async function resolveApiKey(keyHash: string): Promise<CachedApiKey | null> {
  // Try Redis cache first
  const cached = await cacheGet<CachedApiKey>(apiKeyHashKey(keyHash));
  if (cached) return cached;

  // Cache miss — hit Postgres
  const apiKey = await getOne<{
    id: string;
    project_id: string;
    type: 'sdk' | 'management';
    created_by: string;
  }>(
    `SELECT ak.id, ak.project_id, ak.type, ak.created_by
     FROM api_keys ak WHERE ak.key_hash = $1`,
    [keyHash],
  );

  if (!apiKey) return null;

  const user = await getOne<SessionUser>(
    'SELECT id, email, name, role, auth_method FROM users WHERE id = $1',
    [apiKey.created_by],
  );

  if (!user) return null;

  const entry: CachedApiKey = {
    id: apiKey.id,
    project_id: apiKey.project_id,
    type: apiKey.type,
    user,
  };

  // Cache for 5 minutes
  await cacheSet(apiKeyHashKey(keyHash), entry, CACHE_TTL.API_KEY);
  return entry;
}

/**
 * Auth middleware: checks for API key (Bearer token) or session cookie.
 */
export async function authenticate(
  request: FastifyRequest,
  reply: FastifyReply,
): Promise<void> {
  // Try API key auth first (Authorization: Bearer flk_...)
  const authHeader = request.headers.authorization;
  if (authHeader?.startsWith('Bearer ')) {
    const rawKey = authHeader.slice(7);
    const keyHash = hashApiKey(rawKey);

    const resolved = await resolveApiKey(keyHash);
    if (!resolved) {
      throw new UnauthorizedError('Invalid API key');
    }

    // Update last_used_at (fire and forget)
    getOne('UPDATE api_keys SET last_used_at = now() WHERE id = $1', [resolved.id]).catch(() => {});

    request.user = resolved.user;
    request.projectId = resolved.project_id;
    request.apiKeyType = resolved.type;
    return;
  }

  // Try session cookie auth
  const sessionData = request.cookies?.session;
  if (sessionData) {
    try {
      const parsed = JSON.parse(Buffer.from(sessionData, 'base64').toString('utf-8')) as SessionUser;
      const user = await getOne<SessionUser>(
        'SELECT id, email, name, role, auth_method FROM users WHERE id = $1',
        [parsed.id],
      );
      if (user) {
        request.user = user;
        return;
      }
    } catch {
      // Invalid session cookie, fall through
    }
  }

  throw new UnauthorizedError();
}

/**
 * SDK-only auth: requires a valid SDK API key.
 */
export async function authenticateSdk(
  request: FastifyRequest,
  reply: FastifyReply,
): Promise<void> {
  await authenticate(request, reply);
  if (request.apiKeyType !== 'sdk' && request.apiKeyType !== 'management') {
    throw new UnauthorizedError('SDK or management API key required');
  }
}
