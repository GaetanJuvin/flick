import type { Flag, CreateFlagInput, UpdateFlagInput } from '@flick/shared';
import * as repo from './repo.js';
import { NotFoundError, ConflictError } from '../../shared/errors.js';

export async function listFlags(
  projectId: string,
  opts?: { archived?: boolean; tags?: string[] },
): Promise<Flag[]> {
  return repo.findByProject(projectId, opts);
}

export async function getFlag(id: string): Promise<Flag> {
  const flag = await repo.findById(id);
  if (!flag) throw new NotFoundError('Flag', id);
  return flag;
}

export async function getFlagByKey(projectId: string, key: string): Promise<Flag> {
  const flag = await repo.findByKey(projectId, key);
  if (!flag) throw new NotFoundError('Flag', key);
  return flag;
}

export async function createFlag(projectId: string, input: CreateFlagInput): Promise<Flag> {
  const existing = await repo.findByKey(projectId, input.key);
  if (existing) throw new ConflictError(`Flag with key '${input.key}' already exists`);
  return repo.create(projectId, input);
}

export async function updateFlag(id: string, input: UpdateFlagInput): Promise<Flag> {
  const flag = await repo.update(id, input);
  if (!flag) throw new NotFoundError('Flag', id);
  return flag;
}

export async function archiveFlag(id: string): Promise<Flag> {
  const flag = await repo.archive(id);
  if (!flag) throw new NotFoundError('Flag', id);
  return flag;
}

export async function restoreFlag(id: string): Promise<Flag> {
  const flag = await repo.restore(id);
  if (!flag) throw new NotFoundError('Flag', id);
  return flag;
}

export async function deleteFlag(id: string): Promise<void> {
  const deleted = await repo.remove(id);
  if (!deleted) throw new NotFoundError('Flag', id);
}
