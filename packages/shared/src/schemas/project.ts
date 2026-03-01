import { z } from 'zod';

export const createProjectSchema = z.object({
  name: z.string().min(1).max(100),
  slug: z.string().min(1).max(100).regex(/^[a-z0-9-]+$/, 'Slug must be lowercase alphanumeric with hyphens'),
});

export const updateProjectSchema = z.object({
  name: z.string().min(1).max(100).optional(),
});
