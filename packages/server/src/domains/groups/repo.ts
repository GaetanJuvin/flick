import { getOne, getMany } from '../../shared/db.js';
import type { Group, CreateGroupInput, UpdateGroupInput, FlagGroup } from '@flick/shared';

export async function findByProject(projectId: string): Promise<Group[]> {
  return getMany<Group>(
    'SELECT * FROM groups WHERE project_id = $1 ORDER BY name ASC',
    [projectId],
  );
}

export async function findById(id: string): Promise<Group | null> {
  return getOne<Group>('SELECT * FROM groups WHERE id = $1', [id]);
}

export async function findBySlug(projectId: string, slug: string): Promise<Group | null> {
  return getOne<Group>(
    'SELECT * FROM groups WHERE project_id = $1 AND slug = $2',
    [projectId, slug],
  );
}

export async function create(projectId: string, input: CreateGroupInput): Promise<Group> {
  const result = await getOne<Group>(
    `INSERT INTO groups (project_id, name, slug, description, rules)
     VALUES ($1, $2, $3, $4, $5) RETURNING *`,
    [projectId, input.name, input.slug, input.description ?? '', JSON.stringify(input.rules)],
  );
  return result!;
}

export async function update(id: string, input: UpdateGroupInput): Promise<Group | null> {
  const fields: string[] = [];
  const values: unknown[] = [];
  let idx = 1;

  if (input.name !== undefined) { fields.push(`name = $${idx++}`); values.push(input.name); }
  if (input.description !== undefined) { fields.push(`description = $${idx++}`); values.push(input.description); }
  if (input.rules !== undefined) { fields.push(`rules = $${idx++}`); values.push(JSON.stringify(input.rules)); }

  if (fields.length === 0) return findById(id);

  values.push(id);
  return getOne<Group>(
    `UPDATE groups SET ${fields.join(', ')} WHERE id = $${idx} RETURNING *`,
    values,
  );
}

export async function remove(id: string): Promise<boolean> {
  const result = await getOne<Group>('DELETE FROM groups WHERE id = $1 RETURNING id', [id]);
  return result !== null;
}

// Flag-Group associations
export async function findFlagGroups(flagEnvironmentId: string): Promise<FlagGroup[]> {
  return getMany<FlagGroup>(
    'SELECT * FROM flag_groups WHERE flag_environment_id = $1',
    [flagEnvironmentId],
  );
}

export async function findGroupsForFlagEnv(flagEnvironmentId: string): Promise<Group[]> {
  return getMany<Group>(
    `SELECT g.* FROM groups g
     JOIN flag_groups fg ON fg.group_id = g.id
     WHERE fg.flag_environment_id = $1
     ORDER BY g.name ASC`,
    [flagEnvironmentId],
  );
}

export async function addGroupToFlag(flagEnvironmentId: string, groupId: string): Promise<FlagGroup> {
  const result = await getOne<FlagGroup>(
    `INSERT INTO flag_groups (flag_environment_id, group_id)
     VALUES ($1, $2)
     ON CONFLICT (flag_environment_id, group_id) DO NOTHING
     RETURNING *`,
    [flagEnvironmentId, groupId],
  );
  // If conflict, fetch existing
  if (!result) {
    return (await getOne<FlagGroup>(
      'SELECT * FROM flag_groups WHERE flag_environment_id = $1 AND group_id = $2',
      [flagEnvironmentId, groupId],
    ))!;
  }
  return result;
}

export async function removeGroupFromFlag(flagEnvironmentId: string, groupId: string): Promise<boolean> {
  const result = await getOne<FlagGroup>(
    'DELETE FROM flag_groups WHERE flag_environment_id = $1 AND group_id = $2 RETURNING id',
    [flagEnvironmentId, groupId],
  );
  return result !== null;
}
