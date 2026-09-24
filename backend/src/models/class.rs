use chrono::{NaiveDate, NaiveTime};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct Class {
    pub id: Uuid,
    pub date: NaiveDate,
    pub start_time: NaiveTime,
    pub end_time: NaiveTime,
    pub capacity: i32,
    pub coach_id: Option<String>,
    pub workout_id: Option<Uuid>,
    pub created_at: Option<chrono::DateTime<chrono::Utc>>,
}

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct ClassSignup {
    pub id: Uuid,
    pub class_id: Uuid,
    pub user_id: String,
    pub signed_up_at: Option<chrono::DateTime<chrono::Utc>>,
    pub attended: Option<bool>,
}

/// Class with current signup count, used for list endpoint.
#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct ClassWithCount {
    pub id: Uuid,
    pub date: NaiveDate,
    pub start_time: NaiveTime,
    pub end_time: NaiveTime,
    pub capacity: i32,
    pub coach_id: Option<String>,
    pub workout_id: Option<Uuid>,
    pub created_at: Option<chrono::DateTime<chrono::Utc>>,
    pub signup_count: i64,
    pub user_signed_up: bool,
}

/// A signed-up member's info for the class detail view.
#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct ClassMember {
    pub user_id: String,
    pub attended: Option<bool>,
}

/// Full class detail with signed-up members.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ClassDetail {
    pub id: Uuid,
    pub date: NaiveDate,
    pub start_time: NaiveTime,
    pub end_time: NaiveTime,
    pub capacity: i32,
    pub coach_id: Option<String>,
    pub workout_id: Option<Uuid>,
    pub created_at: Option<chrono::DateTime<chrono::Utc>>,
    pub members: Vec<ClassMember>,
}
