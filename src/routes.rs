use std::sync::Arc;

use axum::{Router, routing::get};

use crate::{
    app::AppState,
    controller::{callback, get_unmapped_items, login},
    github::GithubService,
    repository::Postgres,
};

pub fn get_router(shared_state: Arc<AppState<GithubService, Postgres>>) -> axum::Router {
    let router = Router::new();
    router
        .route("/login", get(login))
        .route("/callback", get(callback))
        .route("/org/{org}/project/{id}", get(get_unmapped_items))
        .with_state(shared_state)
}
