use askama::Template;
use axum::{
    extract::Query,
    response::{Html, Redirect},
};
use axum_extra::extract::{CookieJar, cookie::Cookie};
use serde::Deserialize;

static CODE_COOKIE_NAME: &str = "code";

#[derive(Template)]
#[template(path = "login.html")]
struct LoginTemplate {
    title: String,
    code: Option<String>,
}

pub async fn login(jar: CookieJar) -> Html<String> {
    let login_template = LoginTemplate {
        title: "Login".to_string(),
        code: jar.get(CODE_COOKIE_NAME).map(|x| x.to_string()),
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

#[axum::debug_handler]
pub async fn callback(params: Query<CallbackParams>, jar: CookieJar) -> (CookieJar, Redirect) {
    (
        jar.add(Cookie::new(CODE_COOKIE_NAME, params.code.clone())),
        Redirect::temporary("/"),
    )
}

#[cfg(test)]
mod tests {
    use axum::{
        extract::Query,
        response::{IntoResponse, Redirect},
    };
    use axum_extra::extract::CookieJar;

    use crate::controllers::{CallbackParams, callback};

    #[tokio::test]
    async fn callback_sets_cookie_and_redirects() {
        let params = Query(CallbackParams {
            code: "abc123".into(),
        });
        let jar = CookieJar::default();
        let (jar2, redirect): (CookieJar, Redirect) = callback(params, jar).await;

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

        let cookie = jar2.get("code").unwrap();
        assert_eq!(cookie.value(), "abc123");
    }
}
