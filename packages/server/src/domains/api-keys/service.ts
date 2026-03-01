import { randomBytes, createHash } from 'node:crypto';
import type { ApiKey, ApiKeyWithRawKey, CreateApiKeyInput, SessionUser } from '@flick/shared';
import * as repo from './repo.js';
import { NotFoundError } from '../../shared/errors.js';
import { cacheDel, apiKeyHashKey } from '../../shared/cache.js';

function generateApiKey(type: 'sdk' | 'management'): { raw: string; prefix: string; hash: string } {
  const prefix = type === 'sdk' ? 'flk_sdk_' : 'flk_mgmt_';
  const random = randomBytes(24).toString('base64url');
  const raw = prefix + random;
  const hash = createHash('sha256').update(raw).digest('hex');
  return { raw, prefix: raw.slice(0, 12), hash };
}

export async function listApiKeys(projectId: string): Promise<ApiKey[]> {
  return repo.findByProject(projectId);
}

export async function createApiKey(
  projectId: string,
  input: CreateApiKeyInput,
  actor: SessionUser,
): Promise<ApiKeyWithRawKey> {
  const { raw, prefix, hash } = generateApiKey(input.type);

  const apiKey = await repo.create({
    project_id: projectId,
    name: input.name,
    key_prefix: prefix,
    key_hash: hash,
    type: input.type,
    environment_id: input.environment_id ?? null,
    created_by: actor.id,
  });

  return { ...apiKey, raw_key: raw };
}

export async function revokeApiKey(id: string): Promise<void> {
  // Find the key to get its hash for cache invalidation
  const keys = await repo.findByProject(''); // We need to find by ID
  const deleted = await repo.remove(id);
  if (!deleted) throw new NotFoundError('ApiKey', id);
}
