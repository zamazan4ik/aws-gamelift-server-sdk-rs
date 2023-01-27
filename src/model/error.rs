#[derive(thiserror::Error, Debug)]
pub enum Error {
    #[error("Service call failed")]
    ServiceCallFailed(#[from] tokio_tungstenite::tungstenite::Error),

    #[error("Local connection failed")]
    LocalConnectionFailed(#[source] tokio_tungstenite::tungstenite::Error),

    #[error("Game session id is not yet because OnStartGameSession has not been received")]
    GameSessionIdNotSet,

    #[error("Termination time is not yet because OnProcessTerminate has not been received")]
    TerminationTimeNotSet,

    #[error("Bad request: {0}")]
    BadRequest(String),

    #[error("Invalid JSON")]
    InvalidJson(#[from] serde_json::Error),

    #[error("Local connection already closed")]
    LocalConnectionAlreadyClosed,

    #[error("Request unsuccessful with status code {0}: {1}")]
    RequestUnsuccessful(u16, String),

    #[error("Request timeout")]
    RequestTimeout,

    #[error("This request was overwritten by another request with the same id")]
    RequestWasOverwritten,
}
