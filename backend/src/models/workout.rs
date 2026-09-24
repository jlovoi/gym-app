use chrono::{DateTime, NaiveDate, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::Type, PartialEq)]
#[sqlx(type_name = "score_type", rename_all = "snake_case")]
#[serde(rename_all = "snake_case")]
pub enum ScoreType {
    Time,
    Reps,
    Weight,
    RoundsReps,
    Distance,
    Custom,
}

impl ScoreType {
    pub fn order_by_clause(&self) -> &'static str {
        match self {
            ScoreType::Time => "ORDER BY primary_value ASC NULLS LAST",
            ScoreType::RoundsReps => {
                "ORDER BY (data->>'rounds')::int DESC, (data->>'extra_reps')::int DESC"
            }
            _ => "ORDER BY primary_value DESC NULLS LAST",
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct Workout {
    pub id: Uuid,
    pub date: NaiveDate,
    pub description: Option<String>,
    pub score_type: ScoreType,
    pub score_label: Option<String>,
    pub score_config: Option<serde_json::Value>,
    pub created_at: Option<DateTime<Utc>>,
}

#[derive(Debug, Deserialize)]
pub struct CreateWorkout {
    pub date: NaiveDate,
    pub description: Option<String>,
    pub score_type: Option<ScoreType>,
    pub score_label: Option<String>,
    pub score_config: Option<serde_json::Value>,
}

#[derive(Debug, Deserialize)]
pub struct UpdateWorkout {
    pub date: Option<NaiveDate>,
    pub description: Option<String>,
    pub score_type: Option<ScoreType>,
    pub score_label: Option<String>,
    pub score_config: Option<serde_json::Value>,
}
