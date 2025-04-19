use askama::Template;
use axum::response::Html;

#[derive(Template)]
#[template(path = "login.html")]
struct LoginTemplate {
    title: String,
}

pub async fn login() -> Html<String> {
    let login_template = LoginTemplate {
        title: "Login".to_string(),
    };
    Html(
        login_template
            .render()
            .expect("Failed to render login template"),
    )
}
