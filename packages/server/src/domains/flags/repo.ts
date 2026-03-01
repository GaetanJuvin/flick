import { getOne, getMany, query, withTransaction } from '../../shared/db.js';
import type { Flag, CreateFlagInput, UpdateFlagInput } from '@flick/shared';

export async function findByProject(
  projectId: string,
  opts: { archived?: boolean; tags?: string[] } = {},
): Promise<Flag[]> {
  let sql = 'SELECT * FROM flags WHERE project_id = $1';
  const params: unknown[] = [projectId];
  let idx = 2;

  if (opts.archived !== undefined) {
    sql += ` AND archived = $${idx++}`;
    params.push(opts.archived);
  }
  if (opts.tags && opts.tags.length > 0) {
    sql += ` AND tags && $${idx++}`;
    params.push(opts.tags);
  }

  sql += ' ORDER BY created_at DESC';
  return getMany<Flag>(sql, params);
}

export async function findById(id: string): Promise<Flag | null> {
  return getOne<Flag>('SELECT * FROM flags WHERE id = $1', [id]);
}

export async function findByKey(projectId: string, key: string): Promise<Flag | null> {
  return getOne<Flag>('SELECT * FROM flags WHERE project_id = $1 AND key = $2', [projectId, key]);
}

export async function create(projectId: string, input: CreateFlagInput): Promise<Flag> {
  return withTransaction(async (client) => {
    const { rows: [flag] } = await client.query<Flag>(
      `INSERT INTO flags (project_id, key, name, description, gate_type, tags)
       VALUES ($1, $2, $3, $4, $5, $6) RETURNING *`,
      [projectId, input.key, input.name, input.description ?? '', input.gate_type, input.tags ?? []],
    );

    // Auto-create flag_environments for all project environments
    const { rows: envs } = await client.query(
      'SELECT id FROM environments WHERE project_id = $1',
      [projectId],
    );

    const defaultConfig = input.gate_type === 'percentage'
      ? JSON.stringify({ percentage: 0, sticky: true })
      : '{}';

    for (const env of envs) {
      await client.query(
        `INSERT INTO flag_environments (flag_id, environment_id, enabled, gate_config)
         VALUES ($1, $2, false, $3)`,
        [flag.id, env.id, defaultConfig],
      );
    }

    return flag;
  });
}

export async function update(id: string, input: UpdateFlagInput): Promise<Flag | null> {
  const fields: string[] = [];
  const values: unknown[] = [];
  let idx = 1;

  if (input.name !== undefined) { fields.push(`name = $${idx++}`); values.push(input.name); }
  if (input.description !== undefined) { fields.push(`description = $${idx++}`); values.push(input.description); }
  if (input.tags !== undefined) { fields.push(`tags = $${idx++}`); values.push(input.tags); }

  if (fields.length === 0) return findById(id);

  values.push(id);
  return getOne<Flag>(
    `UPDATE flags SET ${fields.join(', ')} WHERE id = $${idx} RETURNING *`,
    values,
  );
}

export async function archive(id: string): Promise<Flag | null> {
  return getOne<Flag>('UPDATE flags SET archived = true WHERE id = $1 RETURNING *', [id]);
}

export async function restore(id: string): Promise<Flag | null> {
  return getOne<Flag>('UPDATE flags SET archived = false WHERE id = $1 RETURNING *', [id]);
}

export async function remove(id: string): Promise<boolean> {
  const result = await getOne<Flag>('DELETE FROM flags WHERE id = $1 RETURNING id', [id]);
  return result !== null;
}
