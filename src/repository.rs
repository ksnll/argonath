use crate::{AppSecrets, model::Session};
use sqlx::{PgPool, postgres::PgPoolOptions};

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

pub struct RepositoryError;

pub trait Repository {
    async fn create_session(
        &self,
        create_session_request: CreateSessionRequest,
    ) -> Result<Session, RepositoryError>;
}

pub struct CreateSessionRequest {
    pub user_id: i32,
    pub access_token: String,
    pub refresh_token: String,
}

impl Repository for Postgres {
    async fn create_session(
        &self,
        create_session_request: CreateSessionRequest,
    ) -> Result<Session, RepositoryError> {
        sqlx::query_as!(
            Session,
            "INSERT INTO sessions( user_id, access_token, refresh_token) 
            VALUES ($1, $2, $3) 
            RETURNING id, user_id,  access_token, refresh_token",
            create_session_request.user_id,
            create_session_request.access_token,
            create_session_request.refresh_token
        )
        .fetch_one(&self.pool)
        .await
        .map_err(|_| RepositoryError)
    }
}
