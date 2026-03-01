import { z } from 'zod';

export const paginationSchema = z.object({
  cursor: z.string().optional(),
  limit: z.number().int().min(1).max(100).default(50),
});

export const uuidParamSchema = z.object({
  id: z.string().uuid(),
});
