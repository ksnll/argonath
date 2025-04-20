use std::{error::Error, fmt::Display, sync::Arc};

use crate::{
    AppSecrets,
    github::{Github, GithubService},
    repository::{Postgres, Repository},
    routes::get_router,
};

pub struct App {
    pub address: String,
    pub port: u32,
}

#[derive(Debug)]
pub enum AppStartError {
    FailedToBind(std::io::Error),
    FailedToStart,
}

pub struct AppState<T: Github, U: Repository> {
    pub secrets: &'static AppSecrets,
    pub github: T,
    pub repository: U,
}

impl Display for AppStartError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            AppStartError::FailedToBind(e) => write!(f, "Failed to bind {}", e),
            AppStartError::FailedToStart => write!(f, "Failed to start app"),
        }
    }
}

impl Error for AppStartError {}

impl App {
    pub fn new(address: String, port: u32) -> App {
        Self { address, port }
    }

    pub async fn run(
        &self,
        secrets: &'static AppSecrets,
        repository: Postgres,
    ) -> Result<(), AppStartError> {
        let listener = tokio::net::TcpListener::bind(format!("{}:{}", self.address, &self.port))
            .await
            .map_err(AppStartError::FailedToBind)?;
        tracing::info!("App started on {}:{}", self.address, self.port);
        let github = GithubService {};
        let shared_state = Arc::new(AppState {
            secrets,
            github,
            repository,
        });
        axum::serve(listener, get_router(shared_state))
            .await
            .map_err(|_| AppStartError::FailedToStart)?;
        Ok(())
    }
}
