import { getOne, getMany } from '../../shared/db.js';
import type { FlagEnvironment, UpdateFlagEnvironmentInput } from '@flick/shared';

export async function findByFlag(flagId: string): Promise<FlagEnvironment[]> {
  return getMany<FlagEnvironment>(
    `SELECT fe.* FROM flag_environments fe
     JOIN environments e ON e.id = fe.environment_id
     WHERE fe.flag_id = $1
     ORDER BY e.sort_order ASC`,
    [flagId],
  );
}

export async function findById(id: string): Promise<FlagEnvironment | null> {
  return getOne<FlagEnvironment>('SELECT * FROM flag_environments WHERE id = $1', [id]);
}

export async function findByFlagAndEnv(flagId: string, envId: string): Promise<FlagEnvironment | null> {
  return getOne<FlagEnvironment>(
    'SELECT * FROM flag_environments WHERE flag_id = $1 AND environment_id = $2',
    [flagId, envId],
  );
}

export async function findByEnv(envId: string): Promise<FlagEnvironment[]> {
  return getMany<FlagEnvironment>(
    'SELECT * FROM flag_environments WHERE environment_id = $1',
    [envId],
  );
}

export async function update(id: string, input: UpdateFlagEnvironmentInput): Promise<FlagEnvironment | null> {
  const fields: string[] = [];
  const values: unknown[] = [];
  let idx = 1;

  if (input.enabled !== undefined) { fields.push(`enabled = $${idx++}`); values.push(input.enabled); }
  if (input.gate_config !== undefined) { fields.push(`gate_config = $${idx++}`); values.push(JSON.stringify(input.gate_config)); }

  if (fields.length === 0) return findById(id);

  values.push(id);
  return getOne<FlagEnvironment>(
    `UPDATE flag_environments SET ${fields.join(', ')} WHERE id = $${idx} RETURNING *`,
    values,
  );
}

export async function toggle(id: string): Promise<FlagEnvironment | null> {
  return getOne<FlagEnvironment>(
    'UPDATE flag_environments SET enabled = NOT enabled WHERE id = $1 RETURNING *',
    [id],
  );
}
