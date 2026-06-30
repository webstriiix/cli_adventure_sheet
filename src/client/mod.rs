mod auth;
mod character;
mod compendium;
mod admin;

use crate::models::ApiErrorResponse;

const DEFAULT_BASE_URL: &str = match option_env!("API_URL") {
    Some(url) => url,
    None => "http://localhost:8080/api/v1",
};

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

    pub fn set_token(&mut self, token: String) {
        self.token = Some(token);
    }

    pub fn clear_token(&mut self) {
        self.token = None;
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
            // Read body text for diagnostics and log it.
            let body_text = match response.text().await {
                Ok(t) => t,
                Err(_) => String::new(),
            };
            // Log to api_error.log for debugging server validation errors
            {
                use std::io::Write;
                if let Ok(mut file) = std::fs::OpenOptions::new().create(true).append(true).open("api_error.log") {
                    let preview = if body_text.len() > 1000 {
                        format!("{}... ({} bytes)", &body_text[..1000], body_text.len())
                    } else {
                        body_text.clone()
                    };
                    let _ = writeln!(file, "[{}] HTTP {} => {}\n", url, status.as_u16(), preview);
                }
            }

            let message = match serde_json::from_str::<ApiErrorResponse>(&body_text) {
                Ok(err) => err.error,
                Err(_) => format!("HTTP {}: {}", status.as_u16(), body_text),
            };
            Err(ApiError::Api {
                status: status.as_u16(),
                message,
            })
        }
    }

    async fn handle_empty_response(&self, response: reqwest::Response) -> Result<(), ApiError> {
        let status = response.status();
        let url = response.url().to_string();
        if status.is_success() {
            Ok(())
        } else {
            // Read body text and log for diagnostics
            let body_text = match response.text().await {
                Ok(t) => t,
                Err(_) => String::new(),
            };
            {
                use std::io::Write;
                if let Ok(mut file) = std::fs::OpenOptions::new().create(true).append(true).open("api_error.log") {
                    let preview = if body_text.len() > 1000 {
                        format!("{}... ({} bytes)", &body_text[..1000], body_text.len())
                    } else {
                        body_text.clone()
                    };
                    let _ = writeln!(file, "[{}] HTTP {} => {}\n", url, status.as_u16(), preview);
                }
            }
            let message = match serde_json::from_str::<ApiErrorResponse>(&body_text) {
                Ok(err) => err.error,
                Err(_) => format!("HTTP {}: {}", status.as_u16(), body_text),
            };
            Err(ApiError::Api {
                status: status.as_u16(),
                message,
            })
        }
    }

    pub async fn check_health(&self) -> bool {
        let url = self.url("/check_health");
        match self.http.get(&url).send().await {
            Ok(resp) => resp.status().is_success(),
            Err(_) => false,
        }
    }
}
