use std::collections::HashMap;

use stripe::{
    CheckoutSession, CheckoutSessionMode, Client, CreateCheckoutSession,
    CreateCheckoutSessionLineItems, CreateCustomer, Customer, CustomerId,
};

use crate::error::AppError;

pub fn client(secret_key: &str) -> Client {
    Client::new(secret_key)
}

pub async fn create_customer(
    client: &Client,
    user_id: &str,
    email: Option<&str>,
    name: Option<&str>,
) -> Result<Customer, AppError> {
    let mut params = CreateCustomer::new();
    params.email = email;
    params.name = name;
    params.metadata = Some(HashMap::from([("user_id".into(), user_id.into())]));

    Customer::create(client, params)
        .await
        .map_err(|e| AppError::Internal(format!("Stripe create customer error: {}", e)))
}

pub async fn create_checkout_session(
    client: &Client,
    customer_id: &str,
    price_id: &str,
    plan_type: &str,
    success_url: &str,
    cancel_url: &str,
) -> Result<CheckoutSession, AppError> {
    let cid: CustomerId = customer_id
        .parse()
        .map_err(|_| AppError::Internal("Invalid Stripe customer ID".into()))?;

    let mut params = CreateCheckoutSession::new();
    params.customer = Some(cid);
    params.mode = Some(CheckoutSessionMode::Subscription);
    params.line_items = Some(vec![CreateCheckoutSessionLineItems {
        price: Some(price_id.into()),
        quantity: Some(1),
        ..Default::default()
    }]);
    params.success_url = Some(success_url);
    params.cancel_url = Some(cancel_url);
    params.metadata = Some(HashMap::from([("plan_type".into(), plan_type.into())]));

    CheckoutSession::create(client, params)
        .await
        .map_err(|e| AppError::Internal(format!("Stripe checkout session error: {}", e)))
}
