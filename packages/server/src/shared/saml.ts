import { SAML } from '@node-saml/node-saml';
import { authConfig } from './config.js';
import type { SamlProfile } from '../domains/users/service.js';

export interface SamlClient {
  generateLoginRequest(): Promise<string>;
  validateCallback(body: Record<string, string>): Promise<SamlProfile>;
}

let cachedClient: SamlClient | null = null;

export function getSamlClient(): SamlClient | null {
  if (cachedClient) return cachedClient;
  if (!authConfig.saml) return null;

  const config = authConfig.saml;
  const saml = new SAML({
    entryPoint: config.entryPoint,
    issuer: config.issuer,
    idpCert: config.idpCert,
    callbackUrl: config.callbackUrl,
    wantAssertionsSigned: true,
    wantAuthnResponseSigned: false,
  });

  cachedClient = {
    async generateLoginRequest(): Promise<string> {
      const url = await saml.getAuthorizeUrlAsync('', undefined, {});
      return url;
    },

    async validateCallback(body: Record<string, string>): Promise<SamlProfile> {
      const { profile } = await saml.validatePostResponseAsync(body);
      if (!profile) {
        throw new Error('SAML assertion did not contain a valid profile');
      }

      const email = (profile as Record<string, unknown>)[config.attrEmail] as string
        ?? profile.nameID;
      const name = (profile as Record<string, unknown>)[config.attrName] as string
        ?? email.split('@')[0];

      return {
        nameId: profile.nameID!,
        issuer: profile.issuer!,
        email,
        name,
      };
    },
  };

  return cachedClient;
}
