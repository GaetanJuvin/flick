import type { Group, CreateGroupInput, UpdateGroupInput, FlagGroup } from '@flick/shared';
import * as repo from './repo.js';
import * as flagEnvRepo from '../flag-environments/repo.js';
import { NotFoundError, ConflictError } from '../../shared/errors.js';
import { invalidateEnvFlags } from '../../shared/cache.js';

export async function listGroups(projectId: string): Promise<Group[]> {
  return repo.findByProject(projectId);
}

export async function getGroup(id: string): Promise<Group> {
  const group = await repo.findById(id);
  if (!group) throw new NotFoundError('Group', id);
  return group;
}

export async function createGroup(projectId: string, input: CreateGroupInput): Promise<Group> {
  const existing = await repo.findBySlug(projectId, input.slug);
  if (existing) throw new ConflictError(`Group with slug '${input.slug}' already exists`);
  return repo.create(projectId, input);
}

export async function updateGroup(id: string, input: UpdateGroupInput): Promise<Group> {
  const group = await repo.update(id, input);
  if (!group) throw new NotFoundError('Group', id);
  return group;
}

export async function deleteGroup(id: string): Promise<void> {
  const deleted = await repo.remove(id);
  if (!deleted) throw new NotFoundError('Group', id);
}

export async function getGroupsForFlagEnv(flagId: string, envId: string): Promise<Group[]> {
  const fe = await flagEnvRepo.findByFlagAndEnv(flagId, envId);
  if (!fe) throw new NotFoundError('FlagEnvironment');
  return repo.findGroupsForFlagEnv(fe.id);
}

export async function addGroupToFlag(flagId: string, envId: string, groupId: string): Promise<FlagGroup> {
  const fe = await flagEnvRepo.findByFlagAndEnv(flagId, envId);
  if (!fe) throw new NotFoundError('FlagEnvironment');

  const group = await repo.findById(groupId);
  if (!group) throw new NotFoundError('Group', groupId);

  const result = await repo.addGroupToFlag(fe.id, groupId);
  await invalidateEnvFlags(envId);
  return result;
}

export async function removeGroupFromFlag(flagId: string, envId: string, groupId: string): Promise<void> {
  const fe = await flagEnvRepo.findByFlagAndEnv(flagId, envId);
  if (!fe) throw new NotFoundError('FlagEnvironment');

  const removed = await repo.removeGroupFromFlag(fe.id, groupId);
  if (!removed) throw new NotFoundError('FlagGroup');
  await invalidateEnvFlags(envId);
}
