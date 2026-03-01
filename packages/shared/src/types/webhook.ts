export type WebhookEvent =
  | 'flag.toggled'
  | 'flag.created'
  | 'flag.updated'
  | 'flag.archived'
  | 'flag.deleted'
  | 'flag.gate_config_updated'
  | 'group.created'
  | 'group.updated'
  | 'group.deleted'
  | 'environment.created'
  | 'environment.updated'
  | 'environment.deleted';

export type WebhookStatus = 'active' | 'inactive';
export type DeliveryStatus = 'success' | 'failure' | 'pending';

export interface Webhook {
  id: string;
  project_id: string;
  url: string;
  secret: string;
  events: WebhookEvent[];
  status: WebhookStatus;
  created_at: string;
  updated_at: string;
}

export interface CreateWebhookInput {
  url: string;
  events: WebhookEvent[];
}

export interface UpdateWebhookInput {
  url?: string;
  events?: WebhookEvent[];
  status?: WebhookStatus;
}

export interface WebhookDelivery {
  id: string;
  webhook_id: string;
  event: WebhookEvent;
  payload: Record<string, unknown>;
  response_status: number | null;
  response_body: string | null;
  status: DeliveryStatus;
  attempted_at: string;
}
