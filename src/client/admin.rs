use super::{ApiClient, ApiError};

impl ApiClient {
    pub async fn bulk_import(&self, json_data: serde_json::Value) -> Result<(), ApiError> {
        let resp = self
            .http
            .post(self.url("/import"))
            .json(&json_data)
            .send()
            .await?;
        self.handle_empty_response(resp).await
    }

    pub async fn import_spell_classes(&self, json_data: serde_json::Value) -> Result<(), ApiError> {
        let resp = self
            .http
            .post(self.url("/import/spell-classes"))
            .json(&json_data)
            .send()
            .await?;
        self.handle_empty_response(resp).await
    }
}
