use reqwest::header;
use serde::Deserialize;
use serde_json::json;

use crate::controller::AppError;

pub struct GithubService {
    pub client: reqwest::Client,
}

#[derive(Deserialize, Debug)]
pub struct OauthResponse {
    pub access_token: String,
    pub refresh_token: String,
}

#[derive(Deserialize, Debug)]
pub struct UserResponse {
    pub login: String,
}

pub trait Github {
    fn new() -> Self;
    async fn post_login_oauth_access_token(
        &self,
        client_id: &str,
        code: &str,
        client_secret: &str,
    ) -> Result<OauthResponse, AppError>;
    async fn get_user(&self, access_token: &str) -> Result<UserResponse, AppError>;
}

impl Github for GithubService {
    fn new() -> GithubService {
        let mut headers = header::HeaderMap::new();
        headers.insert(
            "Accept",
            header::HeaderValue::from_static("application/json"),
        );
        headers.insert(
            "User-Agent",
            header::HeaderValue::from_static("Argonath-App"),
        );
        let client = reqwest::Client::builder()
            .default_headers(headers)
            .build()
            .expect("Failed to build client");
        GithubService { client }
    }
    async fn post_login_oauth_access_token(
        &self,
        client_id: &str,
        code: &str,
        client_secret: &str,
    ) -> Result<OauthResponse, AppError> {
        self.client
            .post("https://github.com/login/oauth/access_token")
            .json(&json!({ "client_id": client_id, "code": code, "client_secret": client_secret }))
            .send()
            .await
            .map_err(|_| AppError)?
            .json::<OauthResponse>()
            .await
            .map_err(|_| AppError)
    }

    async fn get_user(&self, access_token: &str) -> Result<UserResponse, AppError> {
        self.client
            .get("https://api.github.com/user")
            .header("Authorization", format!("Bearer {access_token}"))
            .send()
            .await
            .map_err(|_| AppError)?
            .json::<UserResponse>()
            .await
            .map_err(|_| AppError)
    }
}
