import { z } from 'zod';

export const userRoleSchema = z.enum(['admin', 'viewer']);

export const createUserSchema = z.object({
  email: z.string().email().max(255),
  name: z.string().min(1).max(100),
  password: z.string().min(8).max(128),
  role: userRoleSchema.default('viewer'),
});

export const updateUserSchema = z.object({
  name: z.string().min(1).max(100).optional(),
  email: z.string().email().max(255).optional(),
  role: userRoleSchema.optional(),
});

export const loginSchema = z.object({
  email: z.string().email(),
  password: z.string().min(1),
});

export const updateProfileSchema = z.object({
  name: z.string().min(1).max(100).optional(),
  email: z.string().email().max(255).optional(),
});

export const changePasswordSchema = z.object({
  current_password: z.string().min(1),
  new_password: z.string().min(8).max(128),
});

export const resetPasswordSchema = z.object({
  new_password: z.string().min(8).max(128),
});
