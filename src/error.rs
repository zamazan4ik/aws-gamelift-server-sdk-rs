#[derive(thiserror::Error, Debug, strum_macros::Display)]
pub enum GameLiftErrorType {
    // todo: Make appropriate error type
    ServiceCallFailed,
    LocalConnectionFailed(tokio_tungstenite::tungstenite::Error),
    NetworkNotInitialized,
    GameSessionIdNotSet,
    TerminationTimeNotSet,
    BadRequest,
    InternalServiceError,
    UnexpectedWebSocketMessage,
    InvalidJson(serde_json::Error),
    WebSocketError(tokio_tungstenite::tungstenite::Error),
    WebSocketAlreadyClosed,
    RequestUnsuccessful(u16, String),
    RequestTimeout,
}

impl From<serde_json::Error> for GameLiftErrorType {
    fn from(value: serde_json::Error) -> Self {
        Self::InvalidJson(value)
    }
}

impl From<tokio_tungstenite::tungstenite::Error> for GameLiftErrorType {
    fn from(value: tokio_tungstenite::tungstenite::Error) -> Self {
        Self::WebSocketError(value)
    }
}