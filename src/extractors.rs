use std::sync::Arc;

use axum::{extract::FromRequestParts, http::request::Parts, response::Redirect};
use axum_extra::extract::CookieJar;
use sqlx::types::Uuid;

use crate::{
    app::AppState, controller::SESSION_COOKIE, github::Github, model::Session,
    repository::Repository,
};

pub struct ExtractSession(pub Session);

impl<T, U> FromRequestParts<Arc<AppState<T, U>>> for ExtractSession
where
    T: Github + Send + Sync,
    U: Repository + Send + Sync,
{
    type Rejection = Redirect;

    async fn from_request_parts(
        parts: &mut Parts,
        state: &Arc<AppState<T, U>>,
    ) -> Result<Self, Self::Rejection> {
        let cookies = CookieJar::from_request_parts(parts, state)
            .await
            .expect("Failed to extract cookie");
        let cookie = cookies
            .get(SESSION_COOKIE)
            .ok_or(Redirect::temporary("/login"))?;
        let session_id =
            Uuid::parse_str(cookie.value()).map_err(|_| Redirect::temporary("/login"))?;
        let session = state
            .repository
            .get_session(session_id)
            .await
            .map_err(|_| Redirect::temporary("/login"))?
            .ok_or(Redirect::temporary("/login"))?;
        Ok(ExtractSession(session))
    }
}
