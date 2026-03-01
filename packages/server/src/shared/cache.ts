import { redis } from './redis.js';

export const CACHE_TTL = {
  FLAGS: 60,
  FLAG: 60,
  API_KEY: 300,
} as const;

export function envFlagsKey(envId: string): string {
  return `flick:env:${envId}:flags`;
}

export function envFlagKey(envId: string, flagKey: string): string {
  return `flick:env:${envId}:flag:${flagKey}`;
}

export function apiKeyHashKey(hash: string): string {
  return `flick:apikey:${hash}`;
}

export async function cacheGet<T>(key: string): Promise<T | null> {
  const raw = await redis.get(key);
  if (!raw) return null;
  return JSON.parse(raw) as T;
}

export async function cacheSet(key: string, value: unknown, ttlSeconds: number): Promise<void> {
  await redis.set(key, JSON.stringify(value), 'EX', ttlSeconds);
}

export async function cacheDel(...keys: string[]): Promise<void> {
  if (keys.length > 0) {
    await redis.del(...keys);
  }
}

export async function invalidateEnvFlags(envId: string): Promise<void> {
  // Delete the environment flags list cache
  await cacheDel(envFlagsKey(envId));
  // Also delete individual flag caches for this env using pattern
  const pattern = `flick:env:${envId}:flag:*`;
  const keys = await redis.keys(pattern);
  if (keys.length > 0) {
    await redis.del(...keys);
  }
}
