-- Add auth_method support for SAML SSO
ALTER TABLE users ADD COLUMN auth_method VARCHAR(20) NOT NULL DEFAULT 'password' CHECK (auth_method IN ('password', 'saml'));
ALTER TABLE users ALTER COLUMN password_hash DROP NOT NULL;
ALTER TABLE users ADD COLUMN saml_name_id VARCHAR(255);
ALTER TABLE users ADD COLUMN saml_issuer VARCHAR(500);
CREATE UNIQUE INDEX idx_users_saml_identity ON users (saml_name_id, saml_issuer) WHERE saml_name_id IS NOT NULL;
CREATE INDEX idx_users_auth_method ON users(auth_method);
