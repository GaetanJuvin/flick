import type { FastifyRequest, FastifyReply } from 'fastify';
import { redis } from './redis.js';
import { AppError } from './errors.js';

class RateLimitError extends AppError {
  constructor(retryAfterSec: number, message?: string) {
    super('RATE_LIMITED', message ?? `Too many requests. Retry after ${retryAfterSec} seconds.`, 429);
    this.retryAfter = retryAfterSec;
  }
  retryAfter: number;
}

interface RateLimitOptions {
  /** Redis key prefix */
  prefix: string;
  /** Max requests allowed in the window */
  max: number;
  /** Window size in seconds */
  windowSec: number;
  /** Extract the key to rate-limit by (IP, email, API key hash, etc.) */
  keyFrom: (request: FastifyRequest) => string | null;
  /** Custom message when rate limited */
  message?: string;
}

/**
 * Redis sliding-window rate limiter using INCR + EXPIRE.
 * Returns a Fastify preHandler hook.
 */
export function rateLimit(opts: RateLimitOptions) {
  return async (request: FastifyRequest, reply: FastifyReply): Promise<void> => {
    const id = opts.keyFrom(request);
    if (!id) return; // can't identify → skip

    const key = `flick:rl:${opts.prefix}:${id}`;

    const count = await redis.incr(key);
    if (count === 1) {
      // First request in this window — set TTL
      await redis.expire(key, opts.windowSec);
    }

    const ttl = await redis.ttl(key);
    reply.header('X-RateLimit-Limit', opts.max);
    reply.header('X-RateLimit-Remaining', Math.max(0, opts.max - count));
    reply.header('X-RateLimit-Reset', Math.ceil(Date.now() / 1000) + Math.max(ttl, 0));

    if (count > opts.max) {
      const retryAfter = Math.max(ttl, 1);
      reply.header('Retry-After', retryAfter);
      const msg = opts.message
        ? `${opts.message} Retry after ${retryAfter}s.`
        : `Too many requests. Retry after ${retryAfter} seconds.`;
      throw new RateLimitError(retryAfter, msg);
    }
  };
}

// ── Pre-built limiters ──

/** Login: 5 attempts per IP per 60s */
export const loginRateLimit = rateLimit({
  prefix: 'login',
  max: 5,
  windowSec: 60,
  keyFrom: (req) => req.ip,
  message: 'Whoa, easy there! Too many login attempts. Go grab a coffee and try again.',
});

/** Login by email: 10 attempts per email per 5min (cross-IP brute-force) */
export const loginEmailRateLimit = rateLimit({
  prefix: 'login-email',
  max: 10,
  windowSec: 300,
  keyFrom: (req) => {
    const body = req.body as { email?: string } | undefined;
    return body?.email?.toLowerCase() ?? null;
  },
  message: "That's a lot of password guesses. Maybe try the 'Forgot password' button?",
});

/** SDK evaluate: 1000 requests per API key per 10s (6000 rpm) */
export const evaluateRateLimit = rateLimit({
  prefix: 'eval',
  max: 1000,
  windowSec: 10,
  keyFrom: (req) => {
    const auth = req.headers.authorization;
    if (!auth?.startsWith('Bearer ')) return null;
    // Use the last 8 chars as identifier — no need to hash again
    return auth.slice(-8);
  },
  message: "Dude, you're not supposed to call /evaluate on every request. Use the SDK — it polls and caches flags locally. Read the docs!",
});
