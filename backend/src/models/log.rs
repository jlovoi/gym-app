use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct Log {
    pub id: Uuid,
    pub user_id: Option<String>,
    pub workout_id: Option<Uuid>,
    pub primary_value: Option<f64>,
    pub is_rx: Option<bool>,
    pub data: Option<serde_json::Value>,
    pub created_at: Option<DateTime<Utc>>,
}

/// Log with joined user name, used for leaderboard display.
#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct LeaderboardEntry {
    pub id: Uuid,
    pub user_id: Option<String>,
    pub workout_id: Option<Uuid>,
    pub primary_value: Option<f64>,
    pub is_rx: Option<bool>,
    pub data: Option<serde_json::Value>,
    pub created_at: Option<DateTime<Utc>>,
    pub first_name: Option<String>,
    pub last_name: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct CreateLog {
    pub primary_value: Option<f64>,
    pub is_rx: Option<bool>,
    pub data: Option<serde_json::Value>,
}

#[derive(Debug, Deserialize)]
pub struct UpdateLog {
    pub primary_value: Option<f64>,
    pub is_rx: Option<bool>,
    pub data: Option<serde_json::Value>,
}
