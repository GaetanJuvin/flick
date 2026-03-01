import type { AuditAction, AuditEntityType, AuditEntry, SessionUser } from '@flick/shared';
import * as repo from './repo.js';

export async function log(
  projectId: string,
  actor: SessionUser,
  action: AuditAction,
  entityType: AuditEntityType,
  entityId: string,
  entityName: string,
  before: Record<string, unknown> | null,
  after: Record<string, unknown> | null,
): Promise<AuditEntry> {
  return repo.create({
    project_id: projectId,
    actor_id: actor.id,
    actor_email: actor.email,
    action,
    entity_type: entityType,
    entity_id: entityId,
    entity_name: entityName,
    before,
    after,
  });
}

export async function getAuditLog(
  projectId: string,
  opts?: {
    entity_type?: string;
    entity_id?: string;
    actor_id?: string;
    action?: string;
    cursor?: string;
    limit?: number;
  },
) {
  return repo.findByProject(projectId, opts);
}
