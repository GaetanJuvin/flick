import type { FlagEnvironment, UpdateFlagEnvironmentInput } from '@flick/shared';
import * as repo from './repo.js';
import { NotFoundError } from '../../shared/errors.js';
import { invalidateEnvFlags } from '../../shared/cache.js';

export async function getFlagEnvironments(flagId: string): Promise<FlagEnvironment[]> {
  return repo.findByFlag(flagId);
}

export async function getFlagEnvironment(flagId: string, envId: string): Promise<FlagEnvironment> {
  const fe = await repo.findByFlagAndEnv(flagId, envId);
  if (!fe) throw new NotFoundError('FlagEnvironment');
  return fe;
}

export async function updateFlagEnvironment(
  flagId: string,
  envId: string,
  input: UpdateFlagEnvironmentInput,
): Promise<FlagEnvironment> {
  const fe = await repo.findByFlagAndEnv(flagId, envId);
  if (!fe) throw new NotFoundError('FlagEnvironment');

  const updated = await repo.update(fe.id, input);
  if (!updated) throw new NotFoundError('FlagEnvironment');

  await invalidateEnvFlags(envId);
  return updated;
}

export async function toggleFlagEnvironment(flagId: string, envId: string): Promise<FlagEnvironment> {
  const fe = await repo.findByFlagAndEnv(flagId, envId);
  if (!fe) throw new NotFoundError('FlagEnvironment');

  const toggled = await repo.toggle(fe.id);
  if (!toggled) throw new NotFoundError('FlagEnvironment');

  await invalidateEnvFlags(envId);
  return toggled;
}
