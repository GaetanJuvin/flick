import { getOne, getMany } from '../../shared/db.js';
import type { ApiKey } from '@flick/shared';

interface CreateApiKeyRow {
  project_id: string;
  name: string;
  key_prefix: string;
  key_hash: string;
  type: 'sdk' | 'management';
  environment_id: string | null;
  created_by: string;
}

export async function findByProject(projectId: string): Promise<ApiKey[]> {
  return getMany<ApiKey>(
    'SELECT * FROM api_keys WHERE project_id = $1 ORDER BY created_at DESC',
    [projectId],
  );
}

export async function findByHash(hash: string): Promise<ApiKey | null> {
  return getOne<ApiKey>('SELECT * FROM api_keys WHERE key_hash = $1', [hash]);
}

export async function create(input: CreateApiKeyRow): Promise<ApiKey> {
  const result = await getOne<ApiKey>(
    `INSERT INTO api_keys (project_id, name, key_prefix, key_hash, type, environment_id, created_by)
     VALUES ($1, $2, $3, $4, $5, $6, $7) RETURNING *`,
    [input.project_id, input.name, input.key_prefix, input.key_hash, input.type, input.environment_id, input.created_by],
  );
  return result!;
}

export async function remove(id: string): Promise<boolean> {
  const result = await getOne<{ id: string }>('DELETE FROM api_keys WHERE id = $1 RETURNING id', [id]);
  return result !== null;
}
