import { z } from 'zod';

export const createApiKeySchema = z.object({
  name: z.string().min(1).max(100),
  type: z.enum(['sdk', 'management']),
  environment_id: z.string().uuid().optional(),
});
