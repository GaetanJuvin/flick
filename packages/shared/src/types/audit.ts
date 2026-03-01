export type AuditAction =
  | 'flag.created'
  | 'flag.updated'
  | 'flag.archived'
  | 'flag.restored'
  | 'flag.deleted'
  | 'flag.toggled'
  | 'flag.gate_config_updated'
  | 'flag.group_added'
  | 'flag.group_removed'
  | 'group.created'
  | 'group.updated'
  | 'group.deleted'
  | 'environment.created'
  | 'environment.updated'
  | 'environment.deleted'
  | 'project.created'
  | 'project.updated'
  | 'user.created'
  | 'user.updated'
  | 'user.deleted'
  | 'api_key.created'
  | 'api_key.revoked'
  | 'webhook.created'
  | 'webhook.updated'
  | 'webhook.deleted';

export type AuditEntityType =
  | 'flag'
  | 'group'
  | 'environment'
  | 'project'
  | 'user'
  | 'api_key'
  | 'webhook';

export interface AuditEntry {
  id: string;
  project_id: string;
  actor_id: string;
  actor_email: string;
  action: AuditAction;
  entity_type: AuditEntityType;
  entity_id: string;
  entity_name: string;
  before: Record<string, unknown> | null;
  after: Record<string, unknown> | null;
  created_at: string;
}
