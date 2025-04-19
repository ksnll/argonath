use axum::{Router, routing::get};

use crate::controllers::{callback, login};

pub fn get_router() -> axum::Router {
    let router = Router::new();
    router
        .route("/", get(login))
        .route("/callback", get(callback))
}
