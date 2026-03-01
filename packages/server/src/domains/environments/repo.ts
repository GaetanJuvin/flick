import { getOne, getMany } from '../../shared/db.js';
import type { Environment, CreateEnvironmentInput, UpdateEnvironmentInput } from '@flick/shared';

export async function findByProject(projectId: string): Promise<Environment[]> {
  return getMany<Environment>(
    'SELECT * FROM environments WHERE project_id = $1 ORDER BY sort_order ASC',
    [projectId],
  );
}

export async function findById(id: string): Promise<Environment | null> {
  return getOne<Environment>('SELECT * FROM environments WHERE id = $1', [id]);
}

export async function findBySlug(projectId: string, slug: string): Promise<Environment | null> {
  return getOne<Environment>(
    'SELECT * FROM environments WHERE project_id = $1 AND slug = $2',
    [projectId, slug],
  );
}

export async function create(
  projectId: string,
  input: CreateEnvironmentInput,
): Promise<Environment> {
  const result = await getOne<Environment>(
    `INSERT INTO environments (project_id, name, slug, color, sort_order)
     VALUES ($1, $2, $3, $4, $5) RETURNING *`,
    [projectId, input.name, input.slug, input.color, input.sort_order ?? 0],
  );
  return result!;
}

export async function update(id: string, input: UpdateEnvironmentInput): Promise<Environment | null> {
  const fields: string[] = [];
  const values: unknown[] = [];
  let idx = 1;

  if (input.name !== undefined) { fields.push(`name = $${idx++}`); values.push(input.name); }
  if (input.color !== undefined) { fields.push(`color = $${idx++}`); values.push(input.color); }
  if (input.sort_order !== undefined) { fields.push(`sort_order = $${idx++}`); values.push(input.sort_order); }

  if (fields.length === 0) return findById(id);

  values.push(id);
  return getOne<Environment>(
    `UPDATE environments SET ${fields.join(', ')} WHERE id = $${idx} RETURNING *`,
    values,
  );
}

export async function remove(id: string): Promise<boolean> {
  const result = await getOne<Environment>('DELETE FROM environments WHERE id = $1 RETURNING id', [id]);
  return result !== null;
}
