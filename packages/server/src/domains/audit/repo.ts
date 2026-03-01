import { getMany, getOne } from '../../shared/db.js';
import type { AuditEntry, AuditAction, AuditEntityType } from '@flick/shared';

interface CreateAuditInput {
  project_id: string;
  actor_id: string;
  actor_email: string;
  action: AuditAction;
  entity_type: AuditEntityType;
  entity_id: string;
  entity_name: string;
  before: Record<string, unknown> | null;
  after: Record<string, unknown> | null;
}

export async function create(input: CreateAuditInput): Promise<AuditEntry> {
  const result = await getOne<AuditEntry>(
    `INSERT INTO audit_log (project_id, actor_id, actor_email, action, entity_type, entity_id, entity_name, before_state, after_state)
     VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9) RETURNING *`,
    [
      input.project_id, input.actor_id, input.actor_email,
      input.action, input.entity_type, input.entity_id, input.entity_name,
      input.before ? JSON.stringify(input.before) : null,
      input.after ? JSON.stringify(input.after) : null,
    ],
  );
  return result!;
}

export async function findByProject(
  projectId: string,
  opts: {
    entity_type?: string;
    entity_id?: string;
    actor_id?: string;
    action?: string;
    cursor?: string;
    limit?: number;
  } = {},
): Promise<{ entries: AuditEntry[]; cursor: string | null; has_more: boolean }> {
  const limit = opts.limit ?? 50;
  let sql = 'SELECT * FROM audit_log WHERE project_id = $1';
  const params: unknown[] = [projectId];
  let idx = 2;

  if (opts.entity_type) { sql += ` AND entity_type = $${idx++}`; params.push(opts.entity_type); }
  if (opts.entity_id) { sql += ` AND entity_id = $${idx++}`; params.push(opts.entity_id); }
  if (opts.actor_id) { sql += ` AND actor_id = $${idx++}`; params.push(opts.actor_id); }
  if (opts.action) { sql += ` AND action = $${idx++}`; params.push(opts.action); }
  if (opts.cursor) {
    const decoded = Buffer.from(opts.cursor, 'base64').toString('utf-8');
    sql += ` AND created_at < $${idx++}`;
    params.push(decoded);
  }

  sql += ` ORDER BY created_at DESC LIMIT $${idx}`;
  params.push(limit + 1);

  const entries = await getMany<AuditEntry>(sql, params);
  const has_more = entries.length > limit;
  if (has_more) entries.pop();

  const cursor = entries.length > 0 && has_more
    ? Buffer.from(entries[entries.length - 1].created_at).toString('base64')
    : null;

  return { entries, cursor, has_more };
}
