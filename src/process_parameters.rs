pub type OnStartGameSessionType =
    dyn Fn(crate::entity::GameSession) + std::marker::Send + std::marker::Sync;
pub type OnUpdateGameSessionType =
    dyn Fn(crate::entity::UpdateGameSession) + std::marker::Send + std::marker::Sync;
pub type OnProcessTerminateType = dyn Fn() + std::marker::Send + std::marker::Sync;
pub type OnHealthCheckType = dyn Fn() -> bool + std::marker::Send + std::marker::Sync;

/// This data type contains the set of parameters sent to the GameLift service
/// in a [ProcessReady](crate::api::Api::process_ready) call.
pub struct ProcessParameters {
    /// Name of callback function that the GameLift service invokes to activate
    /// a new game session. GameLift calls this function in response to the
    /// client request CreateGameSession. The callback function takes a
    /// GameSession object (defined in the GameLift Service API Reference).
    pub on_start_game_session: Box<OnStartGameSessionType>,

    /// Name of callback function that the GameLift service invokes to pass an
    /// updated game session object to the server process. GameLift calls this
    /// function when a match backfill request has been processed in order to
    /// provide updated matchmaker data. It passes a GameSession object, a
    /// status update (updateReason), and the match backfill ticket ID.
    pub on_update_game_session: Box<OnUpdateGameSessionType>,

    /// Name of callback function that the GameLift service invokes to force the
    /// server process to shut down. After calling this function, GameLift waits
    /// five minutes for the server process to shut down and respond with a
    /// ProcessEnding() call before it shuts down the server process.
    pub on_process_terminate: Box<OnProcessTerminateType>,

    /// Name of callback function that the GameLift service invokes to request a
    /// health status report from the server process. GameLift calls this
    /// function every 60 seconds. After calling this function GameLift waits 60
    /// seconds for a response, and if none is received. records the server
    /// process as unhealthy.
    pub on_health_check: Box<OnHealthCheckType>,

    /// Port number the server process will listen on for new player
    /// connections. The value must fall into the port range configured for any
    /// fleet deploying this game server build. This port number is included in
    /// game session and player session objects, which game sessions use when
    /// connecting to a server process.
    pub port: i32,

    /// Object with a list of directory paths to game session log files.
    pub log_parameters: crate::log_parameters::LogParameters,
}
