mod auth;
mod character;
mod compendium;

use crate::models::ApiErrorResponse;

const DEFAULT_BASE_URL: &str = "http://localhost:8080/api/v1";

#[derive(Debug, thiserror::Error)]
pub enum ApiError {
    #[error("Request failed: {0}")]
    Request(#[from] reqwest::Error),

    #[error("API error ({status}): {message}")]
    Api { status: u16, message: String },

    #[error("Parse error: {0}")]
    Parse(String),
}
#[derive(Clone)]
pub struct ApiClient {
    http: reqwest::Client,
    base_url: String,
    token: Option<String>,
}

impl ApiClient {
    pub fn new() -> Self {
        Self {
            http: reqwest::Client::new(),
            base_url: DEFAULT_BASE_URL.to_string(),
            token: None,
        }
    }

    pub fn with_base_url(mut self, base_url: &str) -> Self {
        self.base_url = base_url.trim_end_matches('/').to_string();
        self
    }

    pub fn set_token(&mut self, token: String) {
        self.token = Some(token);
    }

    pub fn clear_token(&mut self) {
        self.token = None;
    }

    pub fn has_token(&self) -> bool {
        self.token.is_some()
    }

    fn url(&self, path: &str) -> String {
        format!("{}{}", self.base_url, path)
    }

    fn auth_get(&self, path: &str) -> reqwest::RequestBuilder {
        let mut req = self.http.get(self.url(path));
        if let Some(token) = &self.token {
            req = req.bearer_auth(token);
        }
        req
    }

    fn auth_post(&self, path: &str) -> reqwest::RequestBuilder {
        let mut req = self.http.post(self.url(path));
        if let Some(token) = &self.token {
            req = req.bearer_auth(token);
        }
        req
    }

    fn auth_put(&self, path: &str) -> reqwest::RequestBuilder {
        let mut req = self.http.put(self.url(path));
        if let Some(token) = &self.token {
            req = req.bearer_auth(token);
        }
        req
    }

    fn auth_delete(&self, path: &str) -> reqwest::RequestBuilder {
        let mut req = self.http.delete(self.url(path));
        if let Some(token) = &self.token {
            req = req.bearer_auth(token);
        }
        req
    }

    fn auth_patch(&self, path: &str) -> reqwest::RequestBuilder {
        let mut req = self.http.patch(self.url(path));
        if let Some(token) = &self.token {
            req = req.bearer_auth(token);
        }
        req
    }

    async fn handle_response<T: serde::de::DeserializeOwned>(
        &self,
        response: reqwest::Response,
    ) -> Result<T, ApiError> {
        let status = response.status();
        let url = response.url().to_string();
        if status.is_success() {
            let body = response.text().await?;
            {
                use std::io::Write;
                if let Ok(mut file) = std::fs::OpenOptions::new()
                    .create(true)
                    .append(true)
                    .open("api_debug.log")
                {
                    let preview = if body.len() > 500 {
                        format!("{}... ({} bytes total)", &body[..500], body.len())
                    } else {
                        body.clone()
                    };
                    let _ = writeln!(
                        file,
                        "[{}] {} => {}\n",
                        std::any::type_name::<T>(),
                        url,
                        preview
                    );
                }
            }
            match serde_json::from_str::<T>(&body) {
                Ok(data) => Ok(data),
                Err(e) => {
                    use std::io::Write;
                    if let Ok(mut file) = std::fs::OpenOptions::new()
                        .create(true)
                        .append(true)
                        .open("deserialize_err.log")
                    {
                        let _ = writeln!(
                            file,
                            "Parse error for type {}: {}\n",
                            std::any::type_name::<T>(),
                            e
                        );
                    }
                    Err(ApiError::Parse(e.to_string()))
                }
            }
        } else {
            let message = match response.json::<ApiErrorResponse>().await {
                Ok(err) => err.error,
                Err(_) => format!("HTTP {status}"),
            };
            Err(ApiError::Api {
                status: status.as_u16(),
                message,
            })
        }
    }

    async fn handle_empty_response(&self, response: reqwest::Response) -> Result<(), ApiError> {
        let status = response.status();
        if status.is_success() {
            Ok(())
        } else {
            let message = match response.json::<ApiErrorResponse>().await {
                Ok(err) => err.error,
                Err(_) => format!("HTTP {status}"),
            };
            Err(ApiError::Api {
                status: status.as_u16(),
                message,
            })
        }
    }
}
