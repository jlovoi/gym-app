use axum::extract::State;
use axum::http::HeaderMap;
use axum::routing::{get, post};
use axum::{Json, Router};
use serde::Deserialize;
use serde_json::{json, Value};

use crate::auth::middleware::AuthUser;
use crate::db::{memberships as memberships_db, users as users_db};
use crate::error::AppError;
use crate::state::AppState;
use crate::stripe as stripe_helper;

pub fn router() -> Router<AppState> {
    Router::new()
        .route("/memberships/me", get(get_my_membership))
        .route("/memberships/checkout", post(create_checkout))
}

async fn get_my_membership(
    user: AuthUser,
    State(state): State<AppState>,
) -> Result<Json<Value>, AppError> {
    let membership = memberships_db::find_active_by_user(&state.db, &user.id).await?;
    match membership {
        Some(m) => Ok(Json(json!(m))),
        None => Ok(Json(json!(null))),
    }
}

#[derive(Deserialize)]
struct CheckoutRequest {
    plan_type: String,
}

async fn create_checkout(
    user: AuthUser,
    State(state): State<AppState>,
    headers: HeaderMap,
    Json(body): Json<CheckoutRequest>,
) -> Result<Json<Value>, AppError> {
    let price_id = state
        .config
        .stripe_price_for_plan(&body.plan_type)
        .ok_or_else(|| AppError::BadRequest(format!("Invalid plan type: {}", body.plan_type)))?
        .to_string();

    let client = stripe_helper::client(&state.config.stripe_secret_key);

    // Ensure user has a Stripe customer ID
    let db_user = users_db::find_by_id(&state.db, &user.id)
        .await?
        .ok_or(AppError::NotFound)?;

    let customer_id = match db_user.stripe_customer_id {
        Some(cid) => cid,
        None => {
            let customer =
                stripe_helper::create_customer(&client, &user.id, None, None).await?;
            let cid = customer.id.to_string();
            users_db::set_stripe_customer_id(&state.db, &user.id, &cid).await?;
            cid
        }
    };

    let origin = headers
        .get("origin")
        .or_else(|| headers.get("referer"))
        .and_then(|v| v.to_str().ok())
        .unwrap_or("http://localhost:3000");
    let base = origin.trim_end_matches('/');

    let session = stripe_helper::create_checkout_session(
        &client,
        &customer_id,
        &price_id,
        &body.plan_type,
        &format!("{}/?checkout=success", base),
        &format!("{}/?checkout=cancel", base),
    )
    .await?;

    let url = session.url.unwrap_or_default();
    Ok(Json(json!({ "url": url })))
}
