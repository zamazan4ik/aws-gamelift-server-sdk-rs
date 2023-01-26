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

pub mod unix_time {
    use std::time::{Duration, SystemTime, UNIX_EPOCH};

    use serde::{Deserialize, Deserializer, Serializer};

    #[allow(clippy::cast_possible_truncation, clippy::missing_errors_doc)]
    pub fn serialize<S>(value: &SystemTime, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let value = match value.duration_since(UNIX_EPOCH) {
            Ok(v) => v.as_millis() as i64,
            Err(e) => -(e.duration().as_millis() as i64),
        };
        serializer.serialize_i64(value)
    }

    #[allow(clippy::cast_sign_loss, clippy::missing_errors_doc)]
    pub fn deserialize<'de, D>(deserializer: D) -> Result<SystemTime, D::Error>
    where
        D: Deserializer<'de>,
    {
        match <i64 as Deserialize>::deserialize(deserializer) {
            Ok(v) => {
                if 0 <= v {
                    Ok(UNIX_EPOCH + Duration::from_millis(v as u64))
                } else {
                    Ok(UNIX_EPOCH + Duration::from_millis(-v as u64))
                }
            }
            Err(e) => Err(e),
        }
    }
}
