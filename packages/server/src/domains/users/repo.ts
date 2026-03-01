import { getOne, getMany } from '../../shared/db.js';
import type { User, CreateUserInput, UpdateUserInput, CreateSamlUserInput } from '@flick/shared';

interface UserRow extends User {
  password_hash: string | null;
  saml_name_id: string | null;
  saml_issuer: string | null;
}

const USER_COLUMNS = 'id, email, name, role, auth_method, created_at, updated_at';

export async function findAll(): Promise<User[]> {
  return getMany<User>(`SELECT ${USER_COLUMNS} FROM users ORDER BY created_at DESC`);
}

export async function findById(id: string): Promise<User | null> {
  return getOne<User>(`SELECT ${USER_COLUMNS} FROM users WHERE id = $1`, [id]);
}

export async function findByIdWithHash(id: string): Promise<UserRow | null> {
  return getOne<UserRow>('SELECT * FROM users WHERE id = $1', [id]);
}

export async function findByEmail(email: string): Promise<UserRow | null> {
  return getOne<UserRow>('SELECT * FROM users WHERE email = $1', [email]);
}

export async function findBySamlIdentity(nameId: string, issuer: string): Promise<UserRow | null> {
  return getOne<UserRow>(
    'SELECT * FROM users WHERE saml_name_id = $1 AND saml_issuer = $2',
    [nameId, issuer],
  );
}

export async function create(input: CreateUserInput & { password_hash: string }): Promise<User> {
  const result = await getOne<User>(
    `INSERT INTO users (email, name, password_hash, role, auth_method)
     VALUES ($1, $2, $3, $4, 'password')
     RETURNING ${USER_COLUMNS}`,
    [input.email, input.name, input.password_hash, input.role],
  );
  return result!;
}

export async function createSamlUser(input: CreateSamlUserInput): Promise<User> {
  const result = await getOne<User>(
    `INSERT INTO users (email, name, auth_method, saml_name_id, saml_issuer, role)
     VALUES ($1, $2, 'saml', $3, $4, 'viewer')
     RETURNING ${USER_COLUMNS}`,
    [input.email, input.name, input.saml_name_id, input.saml_issuer],
  );
  return result!;
}

export async function update(id: string, input: UpdateUserInput): Promise<User | null> {
  const fields: string[] = [];
  const values: unknown[] = [];
  let idx = 1;

  if (input.name !== undefined) { fields.push(`name = $${idx++}`); values.push(input.name); }
  if (input.email !== undefined) { fields.push(`email = $${idx++}`); values.push(input.email); }
  if (input.role !== undefined) { fields.push(`role = $${idx++}`); values.push(input.role); }

  if (fields.length === 0) return findById(id);

  values.push(id);
  return getOne<User>(
    `UPDATE users SET ${fields.join(', ')} WHERE id = $${idx}
     RETURNING ${USER_COLUMNS}`,
    values,
  );
}

export async function updateProfile(id: string, input: { name?: string; email?: string }): Promise<User | null> {
  const fields: string[] = [];
  const values: unknown[] = [];
  let idx = 1;

  if (input.name !== undefined) { fields.push(`name = $${idx++}`); values.push(input.name); }
  if (input.email !== undefined) { fields.push(`email = $${idx++}`); values.push(input.email); }

  if (fields.length === 0) return findById(id);

  values.push(id);
  return getOne<User>(
    `UPDATE users SET ${fields.join(', ')} WHERE id = $${idx}
     RETURNING ${USER_COLUMNS}`,
    values,
  );
}

export async function updatePasswordHash(id: string, passwordHash: string): Promise<boolean> {
  const result = await getOne<{ id: string }>(
    'UPDATE users SET password_hash = $1 WHERE id = $2 RETURNING id',
    [passwordHash, id],
  );
  return result !== null;
}

export async function remove(id: string): Promise<boolean> {
  const result = await getOne<{ id: string }>('DELETE FROM users WHERE id = $1 RETURNING id', [id]);
  return result !== null;
}
