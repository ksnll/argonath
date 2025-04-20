use std::sync::Arc;

use crate::{
    github::Github,
    repository::{CreateSessionRequest, Repository},
};
use askama::Template;
use axum::{
    extract::{Query, State},
    http::StatusCode,
    response::{Html, IntoResponse, Redirect, Response},
};
use axum_extra::extract::CookieJar;
use serde::Deserialize;

use crate::app::AppState;

static CODE_COOKIE_NAME: &str = "code";
static ACCESS_TOKEN: &str = "access_token";
static CLIENT_ID: &str = "Iv23li3UZlzZ0kG6gw5s";

#[derive(Debug)]
pub struct AppError;

impl IntoResponse for AppError {
    fn into_response(self) -> Response {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            "Request failed".to_string(),
        )
            .into_response()
    }
}

#[derive(Template)]
#[template(path = "login.html")]
struct LoginTemplate {
    title: String,
    access_token: Option<String>,
    client_id: &'static str,
}

pub async fn login(jar: CookieJar) -> Html<String> {
    let login_template = LoginTemplate {
        title: "Login".to_string(),
        access_token: jar.get(CODE_COOKIE_NAME).map(|x| x.to_string()),
        client_id: CLIENT_ID,
    };
    Html(
        login_template
            .render()
            .expect("Failed to render login template"),
    )
}

#[derive(Deserialize)]
pub struct CallbackParams {
    code: String,
}

pub async fn callback<T: Github, U: Repository>(
    params: Query<CallbackParams>,
    State(state): State<Arc<AppState<T, U>>>,
) -> Result<Redirect, AppError> {
    let res = state
        .github
        .post_login_oauth_access_token(CLIENT_ID, &params.code, &state.secrets.client_secret)
        .await?;
    state.repository.create_session(CreateSessionRequest {
        user_id: 1,
        access_token: res.access_token,
        refresh_token: res.refresh_token,
    });

    Ok(Redirect::temporary("/"))
}

#[cfg(test)]
mod tests {
    use std::sync::Arc;

    use axum::{
        extract::{Query, State},
        response::IntoResponse,
    };
    use axum_extra::extract::CookieJar;

    use crate::{
        AppSecrets,
        app::AppState,
        controller::{CallbackParams, callback},
        github::{Github, OauthResponse, UserResponse},
        model::Session,
        repository::{Repository, RepositoryError},
    };

    struct RepositoryMock;
    impl Repository for RepositoryMock {
        async fn create_session(
            &self,
            _: crate::repository::CreateSessionRequest,
        ) -> Result<crate::model::Session, RepositoryError> {
            Ok(Session {
                id: "id".to_string(),
                user_id: 2,
                access_token: "access_token".to_string(),
                refresh_token: "refresh_token".to_string(),
            })
        }
    }
    struct GithubServiceMock;
    impl Github for GithubServiceMock {
        async fn post_login_oauth_access_token(
            &self,
            _: &str,
            _: &str,
            _: &str,
        ) -> Result<OauthResponse, super::AppError> {
            Ok(OauthResponse {
                access_token: "access_token".to_string(),
                refresh_token: "refresh_token".to_string(),
            })
        }

        async fn get_user(
            &self,
            _: String,
        ) -> Result<crate::github::UserResponse, super::AppError> {
            Ok(UserResponse {
                email: "test@email.com".to_owned(),
            })
        }
    }

    #[tokio::test]
    async fn callback_sets_cookie_and_redirects() {
        let github_mock = GithubServiceMock;
        let repository_mock = RepositoryMock;
        let params = Query(CallbackParams {
            code: "code".to_string(),
        });
        let app_secrets = Box::new(AppSecrets {
            client_secret: "client_secret".to_string(),
            pg_url: "test_url".to_string(),
        });
        let app_secrets = Box::leak(app_secrets);

        let app_state = AppState {
            secrets: app_secrets,
            github: github_mock,
            repository: repository_mock,
        };
        let redirect = callback(params, State(Arc::new(app_state))).await.unwrap();

        assert_eq!(
            redirect
                .into_response()
                .headers()
                .get(axum::http::header::LOCATION)
                .unwrap()
                .to_str()
                .unwrap(),
            "/"
        );
    }
}
