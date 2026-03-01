import { z } from 'zod';

export const evaluationContextSchema = z.object({
  key: z.string().min(1).max(500),
  attributes: z.record(
    z.union([z.string(), z.number(), z.boolean(), z.array(z.string())])
  ).default({}),
});

export const evaluateRequestSchema = z.object({
  flag_key: z.string().min(1),
  context: evaluationContextSchema,
});

export const batchEvaluateRequestSchema = z.object({
  context: evaluationContextSchema,
});
