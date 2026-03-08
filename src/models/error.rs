use serde::{Deserialize, Serialize};

pub type JsonValue = serde_json::Value;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ApiErrorResponse {
    pub error: String,
}
