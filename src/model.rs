use crate::github::DateTime;

#[derive(Debug)]
pub struct Session {
    pub id: String,
    pub user_id: i32,
    pub access_token: String,
    pub refresh_token: String,
    pub expires_at: DateTime,
}

#[derive(Debug)]
pub struct User {
    pub id: i32,
    pub github_login: String,
}
