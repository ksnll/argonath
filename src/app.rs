use std::{error::Error, fmt::Display};

use crate::routes::get_router;

pub struct App {
    pub address: String,
    pub port: u32,
}

#[derive(Debug)]
pub enum AppStartError {
    FailedToBind(std::io::Error),
    FailedToStart,
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

    pub async fn run(&self) -> Result<(), AppStartError> {
        let listener = tokio::net::TcpListener::bind(format!("{}:{}", self.address, &self.port))
            .await
            .map_err(AppStartError::FailedToBind)?;
        tracing::info!("App started on {}:{}", self.address, self.port);
        axum::serve(listener, get_router())
            .await
            .map_err(|_| AppStartError::FailedToStart)?;
        Ok(())
    }
}
