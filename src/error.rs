#[derive(thiserror::Error, Debug, strum_macros::Display)]
pub enum GameLiftErrorType {
    ServiceCallFailed,
    LocalConnectionFailed,
    NetworkNotInitialized,
    GameSessionIdNotSet,
    TerminationTimeNotSet,
    BadRequest,
    InternalServiceError,
    UnexpectedWebSocketMessage,
}
