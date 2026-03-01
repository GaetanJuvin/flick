import { createHash } from 'node:crypto';
import type { EvaluationContext, EvaluationResult, FlagConfig, FullFlagConfig } from '@flick/shared';
import { evaluateFlag } from '@flick/shared';
import { getMany, getOne } from '../../shared/db.js';
import { cacheGet, cacheSet, envFlagsKey, CACHE_TTL } from '../../shared/cache.js';

interface FlagRow {
  key: string;
  gate_type: string;
  enabled: boolean;
  gate_config: Record<string, unknown>;
  archived: boolean;
}

interface GroupRow {
  id: string;
  rules: Array<{ attribute: string; operator: string; value: string | string[] | number }>;
  flag_environment_id: string;
}

async function loadFlagConfigs(envId: string): Promise<FlagConfig[]> {
  // Try cache first
  const cached = await cacheGet<FlagConfig[]>(envFlagsKey(envId));
  if (cached) return cached;

  // Fetch from DB
  const flags = await getMany<FlagRow & { flag_environment_id: string }>(
    `SELECT f.key, f.gate_type, fe.enabled, fe.gate_config, f.archived, fe.id as flag_environment_id
     FROM flags f
     JOIN flag_environments fe ON fe.flag_id = f.id
     WHERE fe.environment_id = $1 AND f.archived = false`,
    [envId],
  );

  // Fetch groups for all flag_environments in this env
  const feIds = flags.map((f) => f.flag_environment_id);
  let groups: GroupRow[] = [];
  if (feIds.length > 0) {
    groups = await getMany<GroupRow>(
      `SELECT fg.flag_environment_id, g.id, g.rules
       FROM flag_groups fg
       JOIN groups g ON g.id = fg.group_id
       WHERE fg.flag_environment_id = ANY($1)`,
      [feIds],
    );
  }

  // Build flag configs
  const groupsByFeId = new Map<string, GroupRow[]>();
  for (const g of groups) {
    const list = groupsByFeId.get(g.flag_environment_id) ?? [];
    list.push(g);
    groupsByFeId.set(g.flag_environment_id, list);
  }

  const configs: FlagConfig[] = flags.map((f) => ({
    key: f.key,
    gate_type: f.gate_type,
    enabled: f.enabled,
    gate_config: f.gate_config,
    groups: (groupsByFeId.get(f.flag_environment_id) ?? []).map((g) => ({
      id: g.id,
      rules: g.rules,
    })),
  }));

  // Cache
  await cacheSet(envFlagsKey(envId), configs, CACHE_TTL.FLAGS);
  return configs;
}

export async function evaluate(
  envId: string,
  flagKey: string,
  context: EvaluationContext,
): Promise<EvaluationResult> {
  const configs = await loadFlagConfigs(envId);
  const flagConfig = configs.find((f) => f.key === flagKey);
  const result = evaluateFlag(flagConfig, context);
  if (!flagConfig) result.flag_key = flagKey;
  return result;
}

export async function evaluateBatch(
  envId: string,
  context: EvaluationContext,
): Promise<Record<string, EvaluationResult>> {
  const configs = await loadFlagConfigs(envId);
  const results: Record<string, EvaluationResult> = {};
  for (const config of configs) {
    results[config.key] = evaluateFlag(config, context);
  }
  return results;
}

export async function getFullConfig(envId: string): Promise<FullFlagConfig> {
  const configs = await loadFlagConfigs(envId);
  const env = await getOne<{ slug: string }>('SELECT slug FROM environments WHERE id = $1', [envId]);

  // Generate a version hash for ETag support
  const hash = createHash('md5').update(JSON.stringify(configs)).digest('hex');

  return {
    environment: env?.slug ?? envId,
    flags: configs,
    version: hash,
  };
}
