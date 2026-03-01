import { z } from 'zod';

export const webhookEventSchema = z.enum([
  'flag.toggled', 'flag.created', 'flag.updated', 'flag.archived', 'flag.deleted', 'flag.gate_config_updated',
  'group.created', 'group.updated', 'group.deleted',
  'environment.created', 'environment.updated', 'environment.deleted',
]);

export const createWebhookSchema = z.object({
  url: z.string().url().max(2000),
  events: z.array(webhookEventSchema).min(1),
});

export const updateWebhookSchema = z.object({
  url: z.string().url().max(2000).optional(),
  events: z.array(webhookEventSchema).min(1).optional(),
  status: z.enum(['active', 'inactive']).optional(),
});
