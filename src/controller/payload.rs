use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
struct PayloadRequest {
    pub persona: String,
    pub platform: String,
    pub identity: String,
    pub patch: serde_json::Value,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct PayloadResponse {
    pub sign_payload: String,
}
