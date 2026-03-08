use crate::models::{AuthResponse, LoginRequest, SignupRequest};

use super::{ApiClient, ApiError};

impl ApiClient {
    pub async fn check_health(&self) -> Result<String, ApiError> {
        let resp = self.http.get(self.url("/check_health")).send().await?;
        let status = resp.status();
        if status.is_success() {
            Ok(resp.text().await?)
        } else {
            Err(ApiError::Api {
                status: status.as_u16(),
                message: "Health check failed".to_string(),
            })
        }
    }

    pub async fn signup(&mut self, req: &SignupRequest) -> Result<AuthResponse, ApiError> {
        let resp = self
            .http
            .post(self.url("/signup"))
            .json(req)
            .send()
            .await?;
        let auth: AuthResponse = self.handle_response(resp).await?;
        self.token = Some(auth.token.clone());
        Ok(auth)
    }

    pub async fn login(&mut self, req: &LoginRequest) -> Result<AuthResponse, ApiError> {
        let resp = self
            .http
            .post(self.url("/login"))
            .json(req)
            .send()
            .await?;
        let auth: AuthResponse = self.handle_response(resp).await?;
        self.token = Some(auth.token.clone());
        Ok(auth)
    }
}
