use chrono::{DateTime, Utc};

#[derive(Debug)]
pub struct ListUrlRedirectModel {
    pub id: i64,
    pub url_path: String,
    pub url_redirect: String,
    pub created_at: DateTime<Utc>,
    pub created_by_user_id: i64,
    pub username: String,
}

#[derive(Debug, Default)]
pub struct GetUrlRedirectModel {
    pub url_path: String,
    pub url_redirect: String,
}

#[derive(Debug, Default)]
pub struct GetUserIdByUrlIdModel {
    pub created_by_user_id: i64,
}
