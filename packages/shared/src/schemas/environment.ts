import { z } from 'zod';

export const createEnvironmentSchema = z.object({
  name: z.string().min(1).max(50),
  slug: z.string().min(1).max(50).regex(/^[a-z0-9-]+$/, 'Slug must be lowercase alphanumeric with hyphens'),
  color: z.string().min(1).max(20),
  sort_order: z.number().int().min(0).optional(),
});

export const updateEnvironmentSchema = z.object({
  name: z.string().min(1).max(50).optional(),
  color: z.string().min(1).max(20).optional(),
  sort_order: z.number().int().min(0).optional(),
});
