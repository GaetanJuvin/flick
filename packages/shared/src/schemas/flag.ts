import { z } from 'zod';

export const gateTypeSchema = z.enum(['boolean', 'percentage', 'group']);

export const percentageGateConfigSchema = z.object({
  percentage: z.number().min(0).max(100),
  sticky: z.boolean().default(true),
});

export const gateConfigSchema = z.union([
  z.object({}),
  percentageGateConfigSchema,
]);

export const createFlagSchema = z.object({
  key: z.string().min(1).max(100).regex(/^[a-z0-9-]+$/, 'Flag key must be lowercase alphanumeric with hyphens'),
  name: z.string().min(1).max(200),
  description: z.string().max(1000).default(''),
  gate_type: gateTypeSchema,
  tags: z.array(z.string().max(50)).max(20).default([]),
});

export const updateFlagSchema = z.object({
  name: z.string().min(1).max(200).optional(),
  description: z.string().max(1000).optional(),
  tags: z.array(z.string().max(50)).max(20).optional(),
});

export const updateFlagEnvironmentSchema = z.object({
  enabled: z.boolean().optional(),
  gate_config: gateConfigSchema.optional(),
});
