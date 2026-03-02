use hmac::{Hmac, Mac};
use rand::Rng;
use sha2::Sha256;
use sqlx::PgPool;
use uuid::Uuid;

use crate::error::AppError;

use super::repo;

type HmacSha256 = Hmac<Sha256>;

// ── Helpers ────────────────────────────────────────────────────────────────

fn generate_secret() -> String {
    let bytes: [u8; 32] = rand::thread_rng().gen();
    bytes.iter().map(|b| format!("{:02x}", b)).collect()
}

fn sign_payload(payload: &str, secret: &str) -> String {
    let mut mac =
        HmacSha256::new_from_slice(secret.as_bytes()).expect("HMAC can take key of any size");
    mac.update(payload.as_bytes());
    let result = mac.finalize().into_bytes();
    result.iter().map(|b| format!("{:02x}", b)).collect()
}

// ── Service functions ──────────────────────────────────────────────────────

pub async fn list_webhooks(
    db: &PgPool,
    project_id: Uuid,
) -> Result<Vec<repo::WebhookRow>, AppError> {
    repo::find_by_project(db, project_id)
        .await
        .map_err(AppError::from)
}

pub async fn get_webhook(db: &PgPool, id: Uuid) -> Result<repo::WebhookRow, AppError> {
    repo::find_by_id(db, id)
        .await?
        .ok_or_else(|| AppError::not_found("Webhook", None))
}

pub async fn create_webhook(
    db: &PgPool,
    project_id: Uuid,
    url: &str,
    events: &[String],
) -> Result<repo::WebhookRow, AppError> {
    let secret = generate_secret();
    repo::create(db, project_id, url, &secret, events)
        .await
        .map_err(AppError::from)
}

pub struct UpdateWebhookInput {
    pub url: Option<String>,
    pub events: Option<Vec<String>>,
    pub status: Option<String>,
}

pub async fn update_webhook(
    db: &PgPool,
    id: Uuid,
    input: UpdateWebhookInput,
) -> Result<repo::WebhookRow, AppError> {
    repo::update(
        db,
        id,
        input.url.as_deref(),
        input.events.as_deref(),
        input.status.as_deref(),
    )
    .await?
    .ok_or_else(|| AppError::not_found("Webhook", None))
}

pub async fn delete_webhook(db: &PgPool, id: Uuid) -> Result<(), AppError> {
    let deleted = repo::remove(db, id).await?;
    if !deleted {
        return Err(AppError::not_found("Webhook", None));
    }
    Ok(())
}

pub async fn get_deliveries(
    db: &PgPool,
    webhook_id: Uuid,
) -> Result<Vec<repo::WebhookDeliveryRow>, AppError> {
    repo::find_deliveries(db, webhook_id, 50)
        .await
        .map_err(AppError::from)
}

pub async fn fire_webhooks(
    db: &PgPool,
    project_id: Uuid,
    event: &str,
    payload: serde_json::Value,
) -> Result<(), AppError> {
    let webhooks = repo::find_by_project_and_event(db, project_id, event).await?;

    for webhook in webhooks {
        let db = db.clone();
        let event = event.to_string();
        let payload = payload.clone();

        tokio::spawn(async move {
            let _ = deliver_webhook(&db, &webhook, &event, &payload).await;
        });
    }

    Ok(())
}

async fn deliver_webhook(
    db: &PgPool,
    webhook: &repo::WebhookRow,
    event: &str,
    payload: &serde_json::Value,
) -> Result<(), AppError> {
    let body = serde_json::to_string(payload).unwrap_or_default();
    let signature = sign_payload(&body, &webhook.secret);

    let client = reqwest::Client::new();
    let response = client
        .post(&webhook.url)
        .header("Content-Type", "application/json")
        .header("X-Flick-Signature", &signature)
        .header("X-Flick-Event", event)
        .timeout(std::time::Duration::from_secs(10))
        .body(body.clone())
        .send()
        .await;

    match response {
        Ok(resp) => {
            let status_code = resp.status().as_u16() as i32;
            let resp_body = resp.text().await.unwrap_or_default();
            let delivery_status = if (200..300).contains(&status_code) {
                "success"
            } else {
                "failed"
            };

            let _ = repo::create_delivery(
                db,
                webhook.id,
                event,
                payload.clone(),
                Some(status_code),
                Some(resp_body),
                delivery_status,
            )
            .await;
        }
        Err(e) => {
            let _ = repo::create_delivery(
                db,
                webhook.id,
                event,
                payload.clone(),
                None,
                Some(e.to_string()),
                "failed",
            )
            .await;
        }
    }

    Ok(())
}

pub async fn test_webhook(db: &PgPool, id: Uuid) -> Result<repo::WebhookDeliveryRow, AppError> {
    let webhook = get_webhook(db, id).await?;

    let test_payload = serde_json::json!({
        "event": "webhook.test",
        "timestamp": chrono::Utc::now().to_rfc3339(),
        "data": {
            "message": "This is a test webhook delivery"
        }
    });

    let body = serde_json::to_string(&test_payload).unwrap_or_default();
    let signature = sign_payload(&body, &webhook.secret);

    let client = reqwest::Client::new();
    let response = client
        .post(&webhook.url)
        .header("Content-Type", "application/json")
        .header("X-Flick-Signature", &signature)
        .header("X-Flick-Event", "webhook.test")
        .timeout(std::time::Duration::from_secs(10))
        .body(body)
        .send()
        .await;

    match response {
        Ok(resp) => {
            let status_code = resp.status().as_u16() as i32;
            let resp_body = resp.text().await.unwrap_or_default();
            let delivery_status = if (200..300).contains(&status_code) {
                "success"
            } else {
                "failed"
            };

            repo::create_delivery(
                db,
                webhook.id,
                "webhook.test",
                test_payload,
                Some(status_code),
                Some(resp_body),
                delivery_status,
            )
            .await
            .map_err(AppError::from)
        }
        Err(e) => repo::create_delivery(
            db,
            webhook.id,
            "webhook.test",
            test_payload,
            None,
            Some(e.to_string()),
            "failed",
        )
        .await
        .map_err(AppError::from),
    }
}
