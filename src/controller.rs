use std::sync::Arc;

use crate::{
    github::Github,
    repository::{CreateSessionRequest, Repository, RepositoryError},
};
use askama::Template;
use axum::{
    extract::{Query, State},
    http::StatusCode,
    response::{Html, IntoResponse, Redirect, Response},
};
use axum_extra::extract::{CookieJar, cookie::Cookie};
use serde::Deserialize;

use crate::app::AppState;

static SESSION_COOKIE: &str = "session";
static CLIENT_ID: &str = "Iv23li3UZlzZ0kG6gw5s";

#[derive(Debug)]
pub struct AppError;

impl From<RepositoryError> for AppError {
    fn from(_: RepositoryError) -> Self {
        AppError
    }
}
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
    session: Option<String>,
    client_id: &'static str,
}

pub async fn login(jar: CookieJar) -> Html<String> {
    let login_template = LoginTemplate {
        title: "Login".to_string(),
        session: jar.get(SESSION_COOKIE).map(|x| x.to_string()),
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
    jar: CookieJar,
    State(state): State<Arc<AppState<T, U>>>,
) -> Result<(CookieJar, Redirect), AppError> {
    let res = state
        .github
        .post_login_oauth_access_token(CLIENT_ID, &params.code, &state.secrets.client_secret)
        .await?;
    let user = state.github.get_user(&res.access_token).await?;
    let user = state.repository.get_or_create_user(&user.email).await?;
    let session = state
        .repository
        .create_session(CreateSessionRequest {
            user_id: user.id,
            access_token: res.access_token,
            refresh_token: res.refresh_token,
        })
        .await?;

    Ok((
        jar.add(Cookie::new(SESSION_COOKIE, session.id)),
        Redirect::temporary("/"),
    ))
}

#[cfg(test)]
mod tests {
    use std::sync::Arc;

    use axum::{
        extract::{Query, State},
        response::IntoResponse,
    };
    use axum_extra::extract::CookieJar;
    use mockall::predicate::eq;

    use crate::{
        AppSecrets,
        app::AppState,
        controller::{CallbackParams, SESSION_COOKIE, callback},
        github::{Github, OauthResponse, UserResponse},
        model::{Session, User},
        repository::{CreateSessionRequest, MockRepository},
    };

    struct MockGithubService;
    impl Github for MockGithubService {
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

        async fn get_user(&self, _: &str) -> Result<crate::github::UserResponse, super::AppError> {
            Ok(UserResponse {
                email: "test@email.com".to_owned(),
            })
        }
    }

    #[tokio::test]
    async fn callback_sets_cookie_and_redirects() {
        let github_mock = MockGithubService;
        let mut repository_mock = MockRepository::new();
        let params = Query(CallbackParams {
            code: "code".to_string(),
        });
        let app_secrets = Box::new(AppSecrets {
            client_secret: "client_secret".to_string(),
            pg_url: "test_url".to_string(),
        });
        let app_secrets = Box::leak(app_secrets);

        repository_mock
            .expect_create_session()
            .with(eq(CreateSessionRequest {
                user_id: 1,
                access_token: "access_token".to_string(),
                refresh_token: "refresh_token".to_string(),
            }))
            .returning(|req| {
                Ok(Session {
                    id: "id".to_string(),
                    user_id: req.user_id,
                    access_token: req.access_token.clone(),
                    refresh_token: req.refresh_token.clone(),
                })
            })
            .times(1);

        repository_mock
            .expect_get_or_create_user()
            .with(eq("test@email.com"))
            .times(1)
            .returning(|email| {
                Ok(User {
                    id: 1,
                    email: email.to_string(),
                })
            });

        let app_state = AppState {
            secrets: app_secrets,
            github: github_mock,
            repository: repository_mock,
        };
        let (jar, redirect) = callback(params, CookieJar::new(), State(Arc::new(app_state)))
            .await
            .unwrap();

        assert_eq!(jar.get(SESSION_COOKIE).unwrap().value(), "id");
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
