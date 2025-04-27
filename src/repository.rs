use std::{error::Error, fmt::Display};

use crate::{
    AppSecrets,
    model::{Session, User},
};
use chrono::{DateTime, Utc};
use mockall::automock;
use sqlx::{PgPool, postgres::PgPoolOptions, types::Uuid};

pub struct Postgres {
    pool: PgPool,
}

impl Postgres {
    pub async fn new(secrets: &AppSecrets) -> Self {
        let pool = PgPoolOptions::new()
            .max_connections(5)
            .connect(&secrets.pg_url)
            .await
            .expect("Failed to connect to postgres");
        Self { pool }
    }
}

#[derive(Debug)]
pub enum RepositoryError {
    FailedToCreateSessionError,
    FailedToCreateUserError,
}
impl Display for RepositoryError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Failed to create session: Database insert failed")
    }
}
impl Error for RepositoryError {}

#[automock]
#[async_trait::async_trait]
pub trait Repository: Send + Sync {
    async fn create_session(
        &self,
        create_session_request: CreateSessionRequest,
    ) -> Result<Session, RepositoryError>;
    async fn get_or_create_user(&self, login: &str) -> Result<User, RepositoryError>;
    async fn get_session(&self, session_id: Uuid) -> Result<Option<Session>, RepositoryError>;
}

#[derive(PartialEq, Debug)]
pub struct CreateSessionRequest {
    pub user_id: i32,
    pub access_token: String,
    pub refresh_token: String,
    pub expires_at: DateTime<Utc>,
}

#[async_trait::async_trait]
impl Repository for Postgres {
    async fn create_session(
        &self,
        create_session_request: CreateSessionRequest,
    ) -> Result<Session, RepositoryError> {
        sqlx::query_as!(
            Session,
            "INSERT INTO sessions(user_id, access_token, refresh_token) 
            VALUES ($1, $2, $3) 
            RETURNING id, user_id,  access_token, refresh_token",
            create_session_request.user_id,
            create_session_request.access_token,
            create_session_request.refresh_token
        )
        .fetch_one(&self.pool)
        .await
        .map_err(|_| RepositoryError::FailedToCreateSessionError)
    }
    async fn get_or_create_user(&self, login: &str) -> Result<User, RepositoryError> {
        sqlx::query_as!(
            User,
            "INSERT INTO users(github_login) 
            VALUES ($1) 
            ON CONFLICT(github_login) DO UPDATE SET email = EXCLUDED.email
            RETURNING id, github_login",
            login,
        )
        .fetch_one(&self.pool)
        .await
        .map_err(|_| RepositoryError::FailedToCreateUserError)
    }
    async fn get_session(&self, session_id: Uuid) -> Result<Option<Session>, RepositoryError> {
        sqlx::query_as!(
            Session,
            "SELECT id, user_id, access_token, refresh_token
            FROM sessions
            WHERE id = $1",
            session_id
        )
        .fetch_optional(&self.pool)
        .await
        .map_err(|_| RepositoryError::FailedToCreateSessionError)
    }
}
