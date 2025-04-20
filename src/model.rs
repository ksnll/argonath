use chrono::{DateTime, Utc};

pub struct Session {
    pub id: String,
    pub user_id: i64,
    pub access_token: String,
    pub refresh_token: String,
}
