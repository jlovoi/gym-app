use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::Type, PartialEq)]
#[sqlx(type_name = "membership_status", rename_all = "snake_case")]
pub enum MembershipStatus {
    Active,
    PastDue,
    Canceled,
    Expired,
}

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct Membership {
    pub id: Uuid,
    pub user_id: Option<String>,
    pub stripe_subscription_id: Option<String>,
    pub status: Option<MembershipStatus>,
    pub plan_type: String,
    pub classes_remaining: Option<i32>,
    pub current_period_end: Option<chrono::DateTime<chrono::Utc>>,
    pub created_at: Option<chrono::DateTime<chrono::Utc>>,
    pub updated_at: Option<chrono::DateTime<chrono::Utc>>,
}
