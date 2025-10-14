use chrono::{DateTime, Utc};

pub struct StackModel {
    #[allow(dead_code)]
    pub id: i64,
    pub error_name: String,
    pub error_summary: String,
    pub error_stack: String,
    pub reported_at: DateTime<Utc>,
}

pub struct ListStackModel {
    pub id: i64,
    pub error_name: String,
    pub error_summary: String,
    pub reported_at: DateTime<Utc>,
}
