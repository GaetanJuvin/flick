import { randomBytes, createHmac } from 'node:crypto';
import type { Webhook, CreateWebhookInput, UpdateWebhookInput, WebhookDelivery } from '@flick/shared';
import * as repo from './repo.js';
import { NotFoundError } from '../../shared/errors.js';

function generateSecret(): string {
  return randomBytes(32).toString('hex');
}

function signPayload(payload: string, secret: string): string {
  return createHmac('sha256', secret).update(payload).digest('hex');
}

export async function listWebhooks(projectId: string): Promise<Webhook[]> {
  return repo.findByProject(projectId);
}

export async function getWebhook(id: string): Promise<Webhook> {
  const webhook = await repo.findById(id);
  if (!webhook) throw new NotFoundError('Webhook', id);
  return webhook;
}

export async function createWebhook(projectId: string, input: CreateWebhookInput): Promise<Webhook> {
  const secret = generateSecret();
  return repo.create(projectId, input, secret);
}

export async function updateWebhook(id: string, input: UpdateWebhookInput): Promise<Webhook> {
  const webhook = await repo.update(id, input);
  if (!webhook) throw new NotFoundError('Webhook', id);
  return webhook;
}

export async function deleteWebhook(id: string): Promise<void> {
  const deleted = await repo.remove(id);
  if (!deleted) throw new NotFoundError('Webhook', id);
}

export async function getDeliveries(webhookId: string): Promise<WebhookDelivery[]> {
  return repo.findDeliveries(webhookId);
}

export async function fireWebhooks(
  projectId: string,
  event: string,
  payload: Record<string, unknown>,
): Promise<void> {
  const webhooks = await repo.findByProjectAndEvent(projectId, event);

  for (const webhook of webhooks) {
    const body = JSON.stringify({ event, data: payload, timestamp: new Date().toISOString() });
    const signature = signPayload(body, webhook.secret);

    try {
      const response = await fetch(webhook.url, {
        method: 'POST',
        headers: {
          'Content-Type': 'application/json',
          'X-Flick-Signature': signature,
          'X-Flick-Event': event,
        },
        body,
        signal: AbortSignal.timeout(10_000),
      });

      const responseBody = await response.text();
      await repo.createDelivery(
        webhook.id, event, payload,
        response.status, responseBody,
        response.ok ? 'success' : 'failure',
      );
    } catch (err) {
      await repo.createDelivery(
        webhook.id, event, payload,
        null, err instanceof Error ? err.message : 'Unknown error',
        'failure',
      );
    }
  }
}

export async function testWebhook(id: string): Promise<WebhookDelivery> {
  const webhook = await getWebhook(id);
  const payload = { test: true, message: 'Test webhook delivery from Flick' };
  const body = JSON.stringify({ event: 'test', data: payload, timestamp: new Date().toISOString() });
  const signature = signPayload(body, webhook.secret);

  try {
    const response = await fetch(webhook.url, {
      method: 'POST',
      headers: {
        'Content-Type': 'application/json',
        'X-Flick-Signature': signature,
        'X-Flick-Event': 'test',
      },
      body,
      signal: AbortSignal.timeout(10_000),
    });

    const responseBody = await response.text();
    return repo.createDelivery(
      webhook.id, 'test', payload,
      response.status, responseBody,
      response.ok ? 'success' : 'failure',
    );
  } catch (err) {
    return repo.createDelivery(
      webhook.id, 'test', payload,
      null, err instanceof Error ? err.message : 'Unknown error',
      'failure',
    );
  }
}
