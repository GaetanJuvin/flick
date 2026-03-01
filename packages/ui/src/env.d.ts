/// <reference types="astro/client" />

interface SessionUser {
  id: string;
  email: string;
  name: string;
  role: 'admin' | 'viewer';
  auth_method: 'password' | 'saml';
}

declare namespace App {
  interface Locals {
    user: SessionUser;
  }
}
