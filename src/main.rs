use app::{App, AppStartError};

mod app;
mod controllers;
mod routes;

#[tokio::main]
async fn main() -> Result<(), AppStartError> {
    tracing_subscriber::fmt::init();
    let app = App::new("localhost".to_string(), 3000);
    app.run().await?;
    Ok(())
}
