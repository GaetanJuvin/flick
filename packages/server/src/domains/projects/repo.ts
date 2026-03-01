import { getOne, getMany, query } from '../../shared/db.js';
import type { Project, CreateProjectInput, UpdateProjectInput } from '@flick/shared';

export async function findAll(): Promise<Project[]> {
  return getMany<Project>('SELECT * FROM projects ORDER BY created_at DESC');
}

export async function findById(id: string): Promise<Project | null> {
  return getOne<Project>('SELECT * FROM projects WHERE id = $1', [id]);
}

export async function findBySlug(slug: string): Promise<Project | null> {
  return getOne<Project>('SELECT * FROM projects WHERE slug = $1', [slug]);
}

export async function create(input: CreateProjectInput): Promise<Project> {
  const result = await getOne<Project>(
    `INSERT INTO projects (name, slug) VALUES ($1, $2) RETURNING *`,
    [input.name, input.slug],
  );
  return result!;
}

export async function update(id: string, input: UpdateProjectInput): Promise<Project | null> {
  const fields: string[] = [];
  const values: unknown[] = [];
  let idx = 1;

  if (input.name !== undefined) {
    fields.push(`name = $${idx++}`);
    values.push(input.name);
  }

  if (fields.length === 0) return findById(id);

  values.push(id);
  return getOne<Project>(
    `UPDATE projects SET ${fields.join(', ')} WHERE id = $${idx} RETURNING *`,
    values,
  );
}
