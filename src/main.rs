use app::{App, AppStartError};
use repository::Postgres;
use secrets::Secrets;
use secrets::b64_to_string;
use serde::Deserialize;

mod app;
mod controller;
mod github;
mod model;
mod repository;
mod routes;
mod secrets;

#[derive(Deserialize, Debug)]
#[serde(rename_all = "UPPERCASE")]
pub struct AppSecrets {
    #[serde(deserialize_with = "b64_to_string")]
    client_secret: String,
    #[serde(deserialize_with = "b64_to_string")]
    pg_url: String,
}

#[tokio::main]
async fn main() -> Result<(), AppStartError> {
    tracing_subscriber::fmt::init();
    let secrets: &AppSecrets = Box::leak(Secrets::load());
    let repository = Postgres::new(secrets).await;
    let app = App::new("localhost".to_string(), 3000);
    app.run(secrets, repository).await?;
    Ok(())
}
