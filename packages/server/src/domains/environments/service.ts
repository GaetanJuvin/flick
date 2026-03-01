import type { Environment, CreateEnvironmentInput, UpdateEnvironmentInput } from '@flick/shared';
import * as repo from './repo.js';
import { NotFoundError, ConflictError } from '../../shared/errors.js';

export async function listEnvironments(projectId: string): Promise<Environment[]> {
  return repo.findByProject(projectId);
}

export async function getEnvironment(id: string): Promise<Environment> {
  const env = await repo.findById(id);
  if (!env) throw new NotFoundError('Environment', id);
  return env;
}

export async function createEnvironment(
  projectId: string,
  input: CreateEnvironmentInput,
): Promise<Environment> {
  const existing = await repo.findBySlug(projectId, input.slug);
  if (existing) throw new ConflictError(`Environment with slug '${input.slug}' already exists`);
  return repo.create(projectId, input);
}

export async function updateEnvironment(id: string, input: UpdateEnvironmentInput): Promise<Environment> {
  const env = await repo.update(id, input);
  if (!env) throw new NotFoundError('Environment', id);
  return env;
}

export async function deleteEnvironment(id: string): Promise<void> {
  const deleted = await repo.remove(id);
  if (!deleted) throw new NotFoundError('Environment', id);
}
