import bcrypt from 'bcrypt';
import type { User, CreateUserInput, UpdateUserInput, UpdateProfileInput, SessionUser } from '@flick/shared';
import * as repo from './repo.js';
import { NotFoundError, ConflictError, UnauthorizedError, ForbiddenError, ValidationError } from '../../shared/errors.js';
import { authConfig } from '../../shared/config.js';

const BCRYPT_ROUNDS = 12;

function toSessionUser(user: User): SessionUser {
  return { id: user.id, email: user.email, name: user.name, role: user.role, auth_method: user.auth_method };
}

export async function listUsers(): Promise<User[]> {
  return repo.findAll();
}

export async function getUser(id: string): Promise<User> {
  const user = await repo.findById(id);
  if (!user) throw new NotFoundError('User', id);
  return user;
}

export async function getProfile(id: string): Promise<User> {
  const user = await repo.findById(id);
  if (!user) throw new NotFoundError('User', id);
  return user;
}

export async function createUser(input: CreateUserInput): Promise<User> {
  if (authConfig.mode === 'saml') {
    throw new ForbiddenError('Cannot create password users when auth mode is SAML-only');
  }

  const existing = await repo.findByEmail(input.email);
  if (existing) throw new ConflictError(`User with email '${input.email}' already exists`);

  const password_hash = await bcrypt.hash(input.password, BCRYPT_ROUNDS);
  return repo.create({ ...input, password_hash });
}

export async function updateUser(id: string, input: UpdateUserInput): Promise<User> {
  const user = await repo.update(id, input);
  if (!user) throw new NotFoundError('User', id);
  return user;
}

export async function updateProfile(id: string, input: UpdateProfileInput): Promise<SessionUser> {
  if (input.email) {
    const existing = await repo.findByEmail(input.email);
    if (existing && existing.id !== id) {
      throw new ConflictError(`User with email '${input.email}' already exists`);
    }
  }

  const user = await repo.updateProfile(id, input);
  if (!user) throw new NotFoundError('User', id);
  return toSessionUser(user);
}

export async function changePassword(id: string, currentPassword: string, newPassword: string): Promise<void> {
  const user = await repo.findByIdWithHash(id);
  if (!user) throw new NotFoundError('User', id);

  if (user.auth_method !== 'password') {
    throw new ForbiddenError('SSO users cannot change password');
  }

  const valid = await bcrypt.compare(currentPassword, user.password_hash!);
  if (!valid) throw new UnauthorizedError('Current password is incorrect');

  const hash = await bcrypt.hash(newPassword, BCRYPT_ROUNDS);
  await repo.updatePasswordHash(id, hash);
}

export async function adminResetPassword(adminId: string, targetId: string, newPassword: string): Promise<void> {
  const target = await repo.findByIdWithHash(targetId);
  if (!target) throw new NotFoundError('User', targetId);

  if (target.auth_method !== 'password') {
    throw new ForbiddenError('Cannot reset password for SSO users');
  }

  const hash = await bcrypt.hash(newPassword, BCRYPT_ROUNDS);
  await repo.updatePasswordHash(targetId, hash);
}

export async function deleteUser(id: string): Promise<void> {
  const deleted = await repo.remove(id);
  if (!deleted) throw new NotFoundError('User', id);
}

export async function login(email: string, password: string): Promise<SessionUser> {
  if (authConfig.mode === 'saml') {
    throw new ForbiddenError('Password login is disabled. Use SSO to sign in.');
  }

  const user = await repo.findByEmail(email);
  if (!user) throw new UnauthorizedError('Invalid email or password');

  if (user.auth_method === 'saml') {
    throw new ForbiddenError('This account uses SSO. Please sign in with your identity provider.');
  }

  const valid = await bcrypt.compare(password, user.password_hash!);
  if (!valid) throw new UnauthorizedError('Invalid email or password');

  return toSessionUser(user);
}

export interface SamlProfile {
  nameId: string;
  issuer: string;
  email: string;
  name: string;
}

export async function loginWithSaml(profile: SamlProfile): Promise<SessionUser> {
  // Try to find by SAML identity first
  const existing = await repo.findBySamlIdentity(profile.nameId, profile.issuer);
  if (existing) {
    // Update name/email if changed in IdP
    const updated = await repo.updateProfile(existing.id, { name: profile.name, email: profile.email });
    return toSessionUser(updated ?? existing);
  }

  // Check for email collision with password user
  const emailUser = await repo.findByEmail(profile.email);
  if (emailUser) {
    throw new ConflictError(
      `An account with email '${profile.email}' already exists with password authentication. Contact an administrator.`,
    );
  }

  // JIT provisioning: create new SAML user
  const newUser = await repo.createSamlUser({
    email: profile.email,
    name: profile.name,
    saml_name_id: profile.nameId,
    saml_issuer: profile.issuer,
  });

  return toSessionUser(newUser);
}
