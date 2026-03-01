import { getOne, getMany } from '../../shared/db.js';
import type { Webhook, CreateWebhookInput, UpdateWebhookInput, WebhookDelivery } from '@flick/shared';

export async function findByProject(projectId: string): Promise<Webhook[]> {
  return getMany<Webhook>(
    'SELECT * FROM webhooks WHERE project_id = $1 ORDER BY created_at DESC',
    [projectId],
  );
}

export async function findById(id: string): Promise<Webhook | null> {
  return getOne<Webhook>('SELECT * FROM webhooks WHERE id = $1', [id]);
}

export async function findByProjectAndEvent(projectId: string, event: string): Promise<Webhook[]> {
  return getMany<Webhook>(
    `SELECT * FROM webhooks WHERE project_id = $1 AND status = 'active' AND $2 = ANY(events)`,
    [projectId, event],
  );
}

export async function create(projectId: string, input: CreateWebhookInput, secret: string): Promise<Webhook> {
  const result = await getOne<Webhook>(
    `INSERT INTO webhooks (project_id, url, secret, events)
     VALUES ($1, $2, $3, $4) RETURNING *`,
    [projectId, input.url, secret, input.events],
  );
  return result!;
}

export async function update(id: string, input: UpdateWebhookInput): Promise<Webhook | null> {
  const fields: string[] = [];
  const values: unknown[] = [];
  let idx = 1;

  if (input.url !== undefined) { fields.push(`url = $${idx++}`); values.push(input.url); }
  if (input.events !== undefined) { fields.push(`events = $${idx++}`); values.push(input.events); }
  if (input.status !== undefined) { fields.push(`status = $${idx++}`); values.push(input.status); }

  if (fields.length === 0) return findById(id);

  values.push(id);
  return getOne<Webhook>(
    `UPDATE webhooks SET ${fields.join(', ')} WHERE id = $${idx} RETURNING *`,
    values,
  );
}

export async function remove(id: string): Promise<boolean> {
  const result = await getOne<{ id: string }>('DELETE FROM webhooks WHERE id = $1 RETURNING id', [id]);
  return result !== null;
}

// Deliveries
export async function createDelivery(
  webhookId: string,
  event: string,
  payload: Record<string, unknown>,
  responseStatus: number | null,
  responseBody: string | null,
  status: 'success' | 'failure' | 'pending',
): Promise<WebhookDelivery> {
  const result = await getOne<WebhookDelivery>(
    `INSERT INTO webhook_deliveries (webhook_id, event, payload, response_status, response_body, status)
     VALUES ($1, $2, $3, $4, $5, $6) RETURNING *`,
    [webhookId, event, JSON.stringify(payload), responseStatus, responseBody, status],
  );
  return result!;
}

export async function findDeliveries(webhookId: string, limit = 50): Promise<WebhookDelivery[]> {
  return getMany<WebhookDelivery>(
    'SELECT * FROM webhook_deliveries WHERE webhook_id = $1 ORDER BY attempted_at DESC LIMIT $2',
    [webhookId, limit],
  );
}
