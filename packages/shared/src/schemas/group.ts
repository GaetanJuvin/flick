import { z } from 'zod';

export const ruleOperatorSchema = z.enum([
  'eq', 'neq', 'in', 'not_in', 'contains',
  'starts_with', 'ends_with', 'gt', 'gte', 'lt', 'lte', 'regex',
]);

export const groupRuleSchema = z.object({
  attribute: z.string().min(1).max(100),
  operator: ruleOperatorSchema,
  value: z.union([
    z.string(),
    z.array(z.string()),
    z.number(),
  ]),
});

export const createGroupSchema = z.object({
  name: z.string().min(1).max(100),
  slug: z.string().min(1).max(100).regex(/^[a-z0-9-]+$/, 'Slug must be lowercase alphanumeric with hyphens'),
  description: z.string().max(1000).default(''),
  rules: z.array(groupRuleSchema).min(1),
});

export const updateGroupSchema = z.object({
  name: z.string().min(1).max(100).optional(),
  description: z.string().max(1000).optional(),
  rules: z.array(groupRuleSchema).min(1).optional(),
});
