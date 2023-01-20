use serde::{de::DeserializeOwned, Deserialize, Serialize};

#[derive(Debug, Serialize)]
#[serde(rename_all = "PascalCase")]
pub struct RequestMessage {
    pub action: String,
    pub request_id: String,

    #[serde(flatten)]
    pub content: serde_json::Value,
}

#[derive(Debug, Default, Deserialize)]
#[serde(rename_all = "PascalCase", default)]
pub struct ResponceMessage {
    pub action: String,
    pub request_id: String,
    pub status_code: u16,
    pub error_message: String,

    #[serde(flatten)]
    pub rest_data: serde_json::Value,
}

impl ResponceMessage {
    pub(crate) fn new_fake_success(action: String, request_id: String) -> Self {
        Self {
            action,
            request_id,
            status_code: tokio_tungstenite::tungstenite::http::StatusCode::OK.as_u16(),
            error_message: String::new(),
            rest_data: serde_json::Value::Null,
        }
    }
}

pub trait RequestContent: Serialize {
    const ACTION_NAME: &'static str;
    type Response: DeserializeOwned;
}
