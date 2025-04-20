use std::sync::Arc;

use axum::{Router, routing::get};

use crate::{
    app::AppState,
    controller::{callback, login},
    github::GithubService,
    repository::Postgres,
};

pub fn get_router(shared_state: Arc<AppState<GithubService, Postgres>>) -> axum::Router {
    let router = Router::new();
    router
        .route("/", get(login))
        .route("/callback", get(callback))
        .with_state(shared_state)
}
