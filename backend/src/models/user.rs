use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::Type, PartialEq)]
#[sqlx(type_name = "user_role", rename_all = "lowercase")]
#[serde(rename_all = "lowercase")]
pub enum UserRole {
    Member,
    Staff,
    Admin,
}

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct User {
    pub id: String,
    pub stripe_customer_id: Option<String>,
    pub role: UserRole,
    pub is_active: bool,
    pub created_at: Option<chrono::DateTime<chrono::Utc>>,
    pub first_name: Option<String>,
    pub last_name: Option<String>,
}
