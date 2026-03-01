export type UserRole = 'admin' | 'viewer';

export type AuthMethod = 'password' | 'saml';

export type AuthMode = 'password' | 'saml' | 'both';

export interface AuthConfig {
  mode: AuthMode;
  saml_enabled: boolean;
  saml_login_url: string | null;
}

export interface User {
  id: string;
  email: string;
  name: string;
  role: UserRole;
  auth_method: AuthMethod;
  created_at: string;
  updated_at: string;
}

export interface CreateUserInput {
  email: string;
  name: string;
  password: string;
  role: UserRole;
}

export interface UpdateUserInput {
  name?: string;
  email?: string;
  role?: UserRole;
}

export interface UpdateProfileInput {
  name?: string;
  email?: string;
}

export interface ChangePasswordInput {
  current_password: string;
  new_password: string;
}

export interface CreateSamlUserInput {
  email: string;
  name: string;
  saml_name_id: string;
  saml_issuer: string;
}

export interface LoginInput {
  email: string;
  password: string;
}

export interface SessionUser {
  id: string;
  email: string;
  name: string;
  role: UserRole;
  auth_method: AuthMethod;
}

export interface UserProject {
  id: string;
  user_id: string;
  project_id: string;
  role: UserRole;
  created_at: string;
}
