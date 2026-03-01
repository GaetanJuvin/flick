import type { AuthMode } from '@flick/shared';

const VALID_AUTH_MODES: AuthMode[] = ['password', 'saml', 'both'];

function readAuthMode(): AuthMode {
  const raw = process.env.AUTH_MODE ?? 'password';
  if (!VALID_AUTH_MODES.includes(raw as AuthMode)) {
    throw new Error(`Invalid AUTH_MODE: "${raw}". Must be one of: ${VALID_AUTH_MODES.join(', ')}`);
  }
  return raw as AuthMode;
}

export interface SamlConfig {
  entryPoint: string;
  issuer: string;
  idpCert: string;
  callbackUrl: string;
  attrEmail: string;
  attrName: string;
}

export interface AppAuthConfig {
  mode: AuthMode;
  saml: SamlConfig | null;
}

function readSamlConfig(): SamlConfig | null {
  const entryPoint = process.env.SAML_ENTRY_POINT;
  const issuer = process.env.SAML_ISSUER;
  const idpCert = process.env.SAML_IDP_CERT;
  const callbackUrl = process.env.SAML_CALLBACK_URL;

  if (!entryPoint || !issuer || !idpCert || !callbackUrl) {
    return null;
  }

  return {
    entryPoint,
    issuer,
    idpCert,
    callbackUrl,
    attrEmail: process.env.SAML_ATTR_EMAIL ?? 'email',
    attrName: process.env.SAML_ATTR_NAME ?? 'name',
  };
}

export const authConfig: AppAuthConfig = {
  mode: readAuthMode(),
  saml: readSamlConfig(),
};
