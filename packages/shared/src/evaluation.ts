import type { EvaluationContext, EvaluationResult, FlagConfig } from './types/evaluation.js';
import type { GroupRule, RuleOperator } from './types/group.js';

/**
 * MurmurHash3 (32-bit) - deterministic hash for percentage rollouts.
 * Produces consistent results across TS and Kotlin SDKs.
 */
export function murmurhash3(key: string, seed: number = 0): number {
  let h1 = seed >>> 0;
  const len = key.length;
  const c1 = 0xcc9e2d51;
  const c2 = 0x1b873593;

  let i = 0;
  while (i + 4 <= len) {
    let k1 =
      (key.charCodeAt(i) & 0xff) |
      ((key.charCodeAt(i + 1) & 0xff) << 8) |
      ((key.charCodeAt(i + 2) & 0xff) << 16) |
      ((key.charCodeAt(i + 3) & 0xff) << 24);

    k1 = Math.imul(k1, c1);
    k1 = (k1 << 15) | (k1 >>> 17);
    k1 = Math.imul(k1, c2);

    h1 ^= k1;
    h1 = (h1 << 13) | (h1 >>> 19);
    h1 = Math.imul(h1, 5) + 0xe6546b64;

    i += 4;
  }

  let k1 = 0;
  switch (len & 3) {
    case 3:
      k1 ^= (key.charCodeAt(i + 2) & 0xff) << 16;
    // falls through
    case 2:
      k1 ^= (key.charCodeAt(i + 1) & 0xff) << 8;
    // falls through
    case 1:
      k1 ^= key.charCodeAt(i) & 0xff;
      k1 = Math.imul(k1, c1);
      k1 = (k1 << 15) | (k1 >>> 17);
      k1 = Math.imul(k1, c2);
      h1 ^= k1;
  }

  h1 ^= len;
  h1 ^= h1 >>> 16;
  h1 = Math.imul(h1, 0x85ebca6b);
  h1 ^= h1 >>> 13;
  h1 = Math.imul(h1, 0xc2b2ae35);
  h1 ^= h1 >>> 16;

  return h1 >>> 0;
}

function matchRule(rule: GroupRule, attributes: Record<string, string | number | boolean | string[]>): boolean {
  const attrValue = attributes[rule.attribute];
  if (attrValue === undefined) return false;

  const op = rule.operator as RuleOperator;

  switch (op) {
    case 'eq':
      return String(attrValue) === String(rule.value);
    case 'neq':
      return String(attrValue) !== String(rule.value);
    case 'in':
      if (Array.isArray(rule.value)) {
        return rule.value.includes(String(attrValue));
      }
      return false;
    case 'not_in':
      if (Array.isArray(rule.value)) {
        return !rule.value.includes(String(attrValue));
      }
      return true;
    case 'contains':
      return String(attrValue).includes(String(rule.value));
    case 'starts_with':
      return String(attrValue).startsWith(String(rule.value));
    case 'ends_with':
      return String(attrValue).endsWith(String(rule.value));
    case 'gt':
      return Number(attrValue) > Number(rule.value);
    case 'gte':
      return Number(attrValue) >= Number(rule.value);
    case 'lt':
      return Number(attrValue) < Number(rule.value);
    case 'lte':
      return Number(attrValue) <= Number(rule.value);
    case 'regex':
      try {
        return new RegExp(String(rule.value)).test(String(attrValue));
      } catch {
        return false;
      }
    default:
      return false;
  }
}

function matchGroup(rules: GroupRule[], attributes: Record<string, string | number | boolean | string[]>): boolean {
  // All rules within a group are ANDed
  return rules.every((rule) => matchRule(rule, attributes));
}

/**
 * Evaluate a single flag against a context.
 * This is the core evaluation logic shared between server and SDK.
 */
export function evaluateFlag(flag: FlagConfig | undefined, context: EvaluationContext): EvaluationResult {
  if (!flag) {
    return {
      flag_key: '',
      enabled: false,
      gate_type: 'boolean',
      reason: 'flag_not_found',
    };
  }

  if (!flag.enabled) {
    return {
      flag_key: flag.key,
      enabled: false,
      gate_type: flag.gate_type,
      reason: 'flag_disabled',
    };
  }

  switch (flag.gate_type) {
    case 'boolean':
      return {
        flag_key: flag.key,
        enabled: true,
        gate_type: 'boolean',
        reason: 'boolean_on',
      };

    case 'percentage': {
      const config = flag.gate_config as { percentage?: number };
      const percentage = config.percentage ?? 0;
      const hash = murmurhash3(flag.key + context.key) % 100;
      const enabled = hash < percentage;
      return {
        flag_key: flag.key,
        enabled,
        gate_type: 'percentage',
        reason: enabled ? 'percentage_match' : 'percentage_miss',
      };
    }

    case 'group': {
      // Groups on a flag are ORed — any matching group enables the flag
      const groups = flag.groups ?? [];
      if (groups.length === 0) {
        return {
          flag_key: flag.key,
          enabled: false,
          gate_type: 'group',
          reason: 'group_miss',
        };
      }

      const matched = groups.some((group) =>
        matchGroup(group.rules as GroupRule[], context.attributes)
      );

      return {
        flag_key: flag.key,
        enabled: matched,
        gate_type: 'group',
        reason: matched ? 'group_match' : 'group_miss',
      };
    }

    default:
      return {
        flag_key: flag.key,
        enabled: false,
        gate_type: flag.gate_type,
        reason: 'flag_not_found',
      };
  }
}
