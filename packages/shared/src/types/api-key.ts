export type ApiKeyType = 'sdk' | 'management';

export interface ApiKey {
  id: string;
  project_id: string;
  name: string;
  key_prefix: string;
  key_hash: string;
  type: ApiKeyType;
  environment_id: string | null;
  created_by: string;
  last_used_at: string | null;
  created_at: string;
}

export interface CreateApiKeyInput {
  name: string;
  type: ApiKeyType;
  environment_id?: string;
}

export interface ApiKeyWithRawKey extends ApiKey {
  raw_key: string;
}
