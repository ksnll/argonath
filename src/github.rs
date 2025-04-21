use serde::Deserialize;
use serde_json::json;

use crate::controller::AppError;

pub struct GithubService;

#[derive(Deserialize, Debug)]
pub struct OauthResponse {
    pub access_token: String,
    pub refresh_token: String,
}

#[derive(Deserialize, Debug)]
pub struct UserResponse {
    pub email: String,
}

pub trait Github {
    async fn post_login_oauth_access_token(
        &self,
        client_id: &str,
        code: &str,
        client_secret: &str,
    ) -> Result<OauthResponse, AppError>;
    async fn get_user(&self, access_token: &str) -> Result<UserResponse, AppError>;
}

impl Github for GithubService {
    async fn post_login_oauth_access_token(
        &self,
        client_id: &str,
        code: &str,
        client_secret: &str,
    ) -> Result<OauthResponse, AppError> {
        let client = reqwest::Client::new();
        client
            .post("https://github.com/login/oauth/access_token")
            .json(&json!({ "client_id": client_id, "code": code, "client_secret": client_secret }))
            .header("Accept", "application/json")
            .send()
            .await
            .map_err(|_| AppError)?
            .json::<OauthResponse>()
            .await
            .map_err(|_| AppError)
    }

    async fn get_user(&self, access_token: &str) -> Result<UserResponse, AppError> {
        let client = reqwest::Client::new();
        client
            .get("https://api.github.com/user")
            .header("Authorization", format!("Bearer {access_token}"))
            .header("Accept", "application/json")
            .send()
            .await
            .map_err(|_| AppError)?
            .json::<UserResponse>()
            .await
            .map_err(|_| AppError)
    }
}
