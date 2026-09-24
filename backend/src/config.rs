use std::env;

#[derive(Clone, Debug)]
pub struct Config {
    pub database_url: String,
    pub clerk_jwks_url: String,
    pub clerk_webhook_secret: String,
    pub stripe_secret_key: String,
    pub stripe_webhook_secret: String,
    pub stripe_price_unlimited: String,
    pub stripe_price_punchcard: String,
    pub stripe_price_dropin: String,
    pub clerk_secret_key: String,
    pub clerk_publishable_key: String,
    pub host: String,
    pub port: u16,
}

impl Config {
    pub fn from_env() -> Self {
        Self {
            database_url: env::var("DATABASE_URL").expect("DATABASE_URL must be set"),
            clerk_jwks_url: env::var("CLERK_JWKS_URL").expect("CLERK_JWKS_URL must be set"),
            clerk_webhook_secret: env::var("CLERK_WEBHOOK_SECRET")
                .expect("CLERK_WEBHOOK_SECRET must be set"),
            stripe_secret_key: env::var("STRIPE_SECRET_KEY")
                .expect("STRIPE_SECRET_KEY must be set"),
            stripe_webhook_secret: env::var("STRIPE_WEBHOOK_SECRET")
                .expect("STRIPE_WEBHOOK_SECRET must be set"),
            stripe_price_unlimited: env::var("STRIPE_PRICE_UNLIMITED")
                .expect("STRIPE_PRICE_UNLIMITED must be set"),
            stripe_price_punchcard: env::var("STRIPE_PRICE_PUNCHCARD")
                .expect("STRIPE_PRICE_PUNCHCARD must be set"),
            stripe_price_dropin: env::var("STRIPE_PRICE_DROPIN")
                .expect("STRIPE_PRICE_DROPIN must be set"),
            clerk_secret_key: env::var("CLERK_SECRET_KEY")
                .expect("CLERK_SECRET_KEY must be set"),
            clerk_publishable_key: env::var("CLERK_PUBLISHABLE_KEY")
                .expect("CLERK_PUBLISHABLE_KEY must be set"),
            host: env::var("HOST").unwrap_or_else(|_| "0.0.0.0".into()),
            port: env::var("PORT")
                .unwrap_or_else(|_| "3000".into())
                .parse()
                .expect("PORT must be a number"),
        }
    }

    pub fn stripe_price_for_plan(&self, plan_type: &str) -> Option<&str> {
        match plan_type {
            "unlimited" => Some(&self.stripe_price_unlimited),
            "punchcard" => Some(&self.stripe_price_punchcard),
            "dropin" => Some(&self.stripe_price_dropin),
            _ => None,
        }
    }
}
