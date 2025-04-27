use graphql_client::{GraphQLQuery, Response};
use reqwest::header;
use serde::{Deserialize, Serialize};
use serde_json::json;
use sqlx::types::chrono::{DateTime as ChronoDateTime, Utc};

use crate::controller::AppError;

static GITHUB_GRAPHQL_URL: &str = "https://api.github.com/graphql";

pub struct GithubService {
    pub client: reqwest::Client,
}

#[derive(Deserialize, Debug)]
pub struct OauthResponse {
    pub access_token: String,
    pub refresh_token: String,
    pub expires_in: i64,
}

#[derive(Deserialize, Debug)]
pub struct UserResponse {
    pub login: String,
}

#[derive(Debug, Serialize)]
pub struct Item {
    pub title: String,
    pub author: String,
    pub url: String,
}

#[derive(Deserialize, Debug)]
pub struct ProjectsResponse {}

pub type DateTime = ChronoDateTime<Utc>;

pub type URI = String;

#[derive(GraphQLQuery)]
#[graphql(
    schema_path = "graphql/schema.docs.graphql",
    query_path = "graphql/project_query.graphql",
    response_derives = "Debug"
)]
pub struct ProjectsQuery;

pub trait Github {
    fn new() -> Self;
    async fn post_login_oauth_access_token(
        &self,
        client_id: &str,
        code: &str,
        client_secret: &str,
    ) -> Result<OauthResponse, AppError>;
    async fn get_user(&self, access_token: &str) -> Result<UserResponse, AppError>;
    async fn get_unmapped_items(
        &self,
        org: String,
        id: u32,
        access_token: &str,
    ) -> Result<Vec<Item>, AppError>;
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

    async fn get_unmapped_items(
        &self,
        org: String,
        id: u32,
        access_token: &str,
    ) -> Result<Vec<Item>, AppError> {
        let mut has_next_page = true;
        let mut unmapped_items = vec![];
        static FETCH_AMOUNT: i64 = 100;
        let mut end_cursor = None;
        while has_next_page {
            let request_body = ProjectsQuery::build_query(projects_query::Variables {
                after: end_cursor.clone(),
                id: id.into(),
                org: org.clone(),
                first: FETCH_AMOUNT,
            });

            let res = self
                .client
                .post(GITHUB_GRAPHQL_URL)
                .header("Authorization", format!("Bearer {access_token}"))
                .json(&request_body)
                .send()
                .await
                .map_err(|_| AppError)?;
            let response_body: Response<projects_query::ResponseData> =
                res.json().await.map_err(|_| AppError)?;

            let items = response_body
                .data
                .and_then(|data| data.organization)
                .and_then(|organization| organization.project_v2)
                .and_then(|project| {
                    end_cursor = project.items.page_info.end_cursor;
                    has_next_page = project.items.page_info.has_next_page;
                    project.items.nodes
                })
                .unwrap_or_default();

            for item in items.iter().flatten() {
                if item.task_type.is_none() {
                    let Some(
                        projects_query::ProjectsQueryOrganizationProjectV2ItemsNodesContent::Issue(
                            issue,
                        ),
                    ) = item.content.as_ref()
                    else {
                        continue;
                    };
                    let Some(author) = issue.author.as_ref() else {
                        continue;
                    };
                    unmapped_items.push(Item {
                        title: issue.title.clone(),
                        author: author.login.clone(),
                        url: issue.url.clone(),
                    });
                }
            }
        }
        Ok(unmapped_items)
    }
}
