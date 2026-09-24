use axum::body::Bytes;
use axum::extract::State;
use axum::http::{HeaderMap, StatusCode};
use axum::routing::post;
use axum::Router;
use serde::Deserialize;
use stripe::{EventObject, EventType, Webhook};

use crate::db::{memberships as memberships_db, users as users_db};
use crate::state::AppState;

pub fn router() -> Router<AppState> {
    Router::new()
        .route("/webhooks/clerk", post(clerk_webhook))
        .route("/webhooks/stripe", post(stripe_webhook))
}

// ─── Clerk Webhook ───────────────────────────────────────────────────

#[derive(Deserialize)]
struct ClerkWebhookPayload {
    #[serde(rename = "type")]
    event_type: String,
    data: ClerkEventData,
}

#[derive(Deserialize)]
struct ClerkEventData {
    id: String,
    first_name: Option<String>,
    last_name: Option<String>,
}

async fn clerk_webhook(
    State(state): State<AppState>,
    headers: HeaderMap,
    body: Bytes,
) -> StatusCode {
    if let Err(e) = verify_clerk_signature(
        &headers,
        &body,
        &state.config.clerk_webhook_secret,
    ) {
        tracing::warn!("Clerk webhook signature verification failed: {}", e);
        return StatusCode::UNAUTHORIZED;
    }

    let payload: ClerkWebhookPayload = match serde_json::from_slice(&body) {
        Ok(p) => p,
        Err(e) => {
            tracing::warn!("Failed to parse Clerk webhook: {}", e);
            return StatusCode::BAD_REQUEST;
        }
    };

    match payload.event_type.as_str() {
        "user.created" => {
            if let Err(e) = users_db::create(
                &state.db,
                &payload.data.id,
                payload.data.first_name.as_deref(),
                payload.data.last_name.as_deref(),
            )
            .await
            {
                tracing::error!("Failed to create user from Clerk webhook: {}", e);
                return StatusCode::INTERNAL_SERVER_ERROR;
            }
        }
        "user.deleted" => {
            if let Err(e) = users_db::delete(&state.db, &payload.data.id).await {
                tracing::error!("Failed to delete user from Clerk webhook: {}", e);
                return StatusCode::INTERNAL_SERVER_ERROR;
            }
        }
        _ => {
            tracing::debug!("Ignoring Clerk event: {}", payload.event_type);
        }
    }

    StatusCode::OK
}

fn verify_clerk_signature(
    headers: &HeaderMap,
    body: &[u8],
    secret: &str,
) -> Result<(), String> {
    use base64::Engine;
    use hmac::{Hmac, Mac};
    use sha2::Sha256;

    let msg_id = headers
        .get("svix-id")
        .and_then(|v| v.to_str().ok())
        .ok_or("Missing svix-id header")?;
    let timestamp = headers
        .get("svix-timestamp")
        .and_then(|v| v.to_str().ok())
        .ok_or("Missing svix-timestamp header")?;
    let signature = headers
        .get("svix-signature")
        .and_then(|v| v.to_str().ok())
        .ok_or("Missing svix-signature header")?;

    // Svix secret is base64-encoded after "whsec_" prefix
    let secret_bytes = if let Some(stripped) = secret.strip_prefix("whsec_") {
        base64::engine::general_purpose::STANDARD
            .decode(stripped)
            .map_err(|e| format!("Invalid secret encoding: {}", e))?
    } else {
        secret.as_bytes().to_vec()
    };

    let body_str = std::str::from_utf8(body).unwrap_or("");
    let to_sign = format!("{}.{}.{}", msg_id, timestamp, body_str);

    let mut mac = Hmac::<Sha256>::new_from_slice(&secret_bytes)
        .map_err(|e| format!("HMAC error: {}", e))?;
    mac.update(to_sign.as_bytes());
    let expected = mac.finalize().into_bytes();
    let expected_b64 = base64::engine::general_purpose::STANDARD.encode(expected);

    // Svix sends space-separated signatures like "v1,<base64> v1,<base64>"
    let valid = signature.split(' ').any(|sig| {
        sig.strip_prefix("v1,")
            .map(|s| s == expected_b64)
            .unwrap_or(false)
    });

    if valid {
        Ok(())
    } else {
        Err("Signature mismatch".into())
    }
}

// ─── Stripe Webhook ──────────────────────────────────────────────────

async fn stripe_webhook(
    State(state): State<AppState>,
    headers: HeaderMap,
    body: Bytes,
) -> StatusCode {
    let sig = match headers.get("stripe-signature").and_then(|v| v.to_str().ok()) {
        Some(s) => s.to_string(),
        None => return StatusCode::BAD_REQUEST,
    };

    let payload = match std::str::from_utf8(&body) {
        Ok(s) => s,
        Err(_) => return StatusCode::BAD_REQUEST,
    };

    // Check the event type from raw JSON before full deserialization,
    // since the stripe crate can't parse all event types.
    let event_type = match serde_json::from_str::<serde_json::Value>(payload) {
        Ok(v) => v["type"].as_str().unwrap_or("").to_string(),
        Err(_) => return StatusCode::BAD_REQUEST,
    };

    let handled_types = [
        "checkout.session.completed",
        "customer.subscription.updated",
        "customer.subscription.deleted",
    ];
    if !handled_types.contains(&event_type.as_str()) {
        tracing::debug!("Ignoring Stripe event: {}", event_type);
        return StatusCode::OK;
    }

    let event = match Webhook::construct_event(payload, &sig, &state.config.stripe_webhook_secret) {
        Ok(e) => e,
        Err(e) => {
            tracing::warn!("Stripe webhook verification failed: {}", e);
            return StatusCode::UNAUTHORIZED;
        }
    };

    match event.type_ {
        EventType::CheckoutSessionCompleted => {
            handle_checkout_completed(&state, event.data.object).await
        }
        EventType::CustomerSubscriptionUpdated => {
            handle_subscription_updated(&state, event.data.object).await
        }
        EventType::CustomerSubscriptionDeleted => {
            handle_subscription_deleted(&state, event.data.object).await
        }
        _ => StatusCode::OK,
    }
}

async fn handle_checkout_completed(state: &AppState, object: EventObject) -> StatusCode {
    let session = match object {
        EventObject::CheckoutSession(s) => s,
        _ => return StatusCode::BAD_REQUEST,
    };

    let customer_id = match &session.customer {
        Some(expandable) => expandable.id().to_string(),
        None => {
            tracing::warn!("checkout.session.completed missing customer");
            return StatusCode::BAD_REQUEST;
        }
    };

    let subscription_id = match &session.subscription {
        Some(expandable) => expandable.id().to_string(),
        None => {
            tracing::warn!("checkout.session.completed missing subscription");
            return StatusCode::BAD_REQUEST;
        }
    };

    let plan_type = session
        .metadata
        .as_ref()
        .and_then(|m: &std::collections::HashMap<String, String>| m.get("plan_type"))
        .map(|s: &String| s.as_str())
        .unwrap_or("unlimited");

    // Find user by stripe customer ID
    let user = sqlx::query_as::<_, crate::models::user::User>(
        "SELECT * FROM users WHERE stripe_customer_id = $1",
    )
    .bind(&customer_id)
    .fetch_optional(&state.db)
    .await;

    let user = match user {
        Ok(Some(u)) => u,
        Ok(None) => {
            tracing::warn!("No user found for Stripe customer {}", customer_id);
            return StatusCode::OK;
        }
        Err(e) => {
            tracing::error!("DB error looking up customer: {}", e);
            return StatusCode::INTERNAL_SERVER_ERROR;
        }
    };

    if let Err(e) =
        memberships_db::upsert_from_checkout(&state.db, &user.id, &subscription_id, plan_type)
            .await
    {
        tracing::error!("Failed to create membership: {}", e);
        return StatusCode::INTERNAL_SERVER_ERROR;
    }

    // Activate the user now that they have a membership
    if let Err(e) = users_db::update(&state.db, &user.id, None, Some(true)).await {
        tracing::error!("Failed to activate user after checkout: {}", e);
    }

    StatusCode::OK
}

async fn handle_subscription_updated(state: &AppState, object: EventObject) -> StatusCode {
    let sub = match object {
        EventObject::Subscription(s) => s,
        _ => return StatusCode::BAD_REQUEST,
    };

    let subscription_id = sub.id.to_string();
    let mapped_status = match sub.status {
        stripe::SubscriptionStatus::Active => "active",
        stripe::SubscriptionStatus::PastDue => "past_due",
        stripe::SubscriptionStatus::Canceled => "canceled",
        _ => "expired",
    };

    if let Err(e) =
        memberships_db::update_status_by_subscription(&state.db, &subscription_id, mapped_status)
            .await
    {
        tracing::error!("Failed to update membership status: {}", e);
        return StatusCode::INTERNAL_SERVER_ERROR;
    }

    StatusCode::OK
}

async fn handle_subscription_deleted(state: &AppState, object: EventObject) -> StatusCode {
    let sub = match object {
        EventObject::Subscription(s) => s,
        _ => return StatusCode::BAD_REQUEST,
    };

    let subscription_id = sub.id.to_string();

    if let Err(e) =
        memberships_db::update_status_by_subscription(&state.db, &subscription_id, "canceled")
            .await
    {
        tracing::error!("Failed to cancel membership: {}", e);
        return StatusCode::INTERNAL_SERVER_ERROR;
    }

    StatusCode::OK
}
