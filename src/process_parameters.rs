use std::{future::Future, pin::Pin};

use crate::model::{GameSession, UpdateGameSession};

pub trait GameLiftEventCallbacks
where
    Self: Send,
{
    fn on_start_game_session(
        &mut self,
        game_session: GameSession,
    ) -> Pin<Box<dyn Future<Output = ()> + Send>>;
    fn on_update_game_session(
        &mut self,
        update_game_session: UpdateGameSession,
    ) -> Pin<Box<dyn Future<Output = ()> + Send>>;
    fn on_process_terminate(&mut self) -> Pin<Box<dyn Future<Output = ()> + Send>>;
    fn on_health_check(&mut self) -> Pin<Box<dyn Future<Output = bool> + Send>>;
}

/// This data type contains the set of parameters sent to the GameLift service
/// in a [ProcessReady](crate::api::Api::process_ready) call.
pub struct ProcessParameters<Fn1, Fn2, Fn3, Fn4>
where
    Fn1: 'static,
    Fn2: 'static,
    Fn3: 'static,
    Fn4: 'static,
{
    /// Name of callback function that the GameLift service invokes to activate
    /// a new game session. GameLift calls this function in response to the
    /// client request CreateGameSession. The callback function takes a
    /// GameSession object (defined in the GameLift Service API Reference).
    pub on_start_game_session: Fn1,

    /// Name of callback function that the GameLift service invokes to pass an
    /// updated game session object to the server process. GameLift calls this
    /// function when a match backfill request has been processed in order to
    /// provide updated matchmaker data. It passes a GameSession object, a
    /// status update (updateReason), and the match backfill ticket ID.
    pub on_update_game_session: Fn2,

    /// Name of callback function that the GameLift service invokes to force the
    /// server process to shut down. After calling this function, GameLift waits
    /// five minutes for the server process to shut down and respond with a
    /// ProcessEnding() call before it shuts down the server process.
    pub on_process_terminate: Fn3,

    /// Name of callback function that the GameLift service invokes to request a
    /// health status report from the server process. GameLift calls this
    /// function every 60 seconds. After calling this function GameLift waits 60
    /// seconds for a response, and if none is received. records the server
    /// process as unhealthy.
    pub on_health_check: Fn4,

    /// Port number the server process will listen on for new player
    /// connections. The value must fall into the port range configured for any
    /// fleet deploying this game server build. This port number is included in
    /// game session and player session objects, which game sessions use when
    /// connecting to a server process.
    pub port: u16,

    /// Object with a list of directory paths to game session log files.
    pub log_parameters: crate::log_parameters::LogParameters,
}

impl<Fn1, Fut1, Fn2, Fut2, Fn3, Fut3, Fn4, Fut4> GameLiftEventCallbacks
    for ProcessParameters<Fn1, Fn2, Fn3, Fn4>
where
    Fn1: FnMut(GameSession) -> Fut1 + Send + Sync,
    Fut1: Future<Output = ()> + Send + 'static,
    Fn2: FnMut(UpdateGameSession) -> Fut2 + Send + Sync,
    Fut2: Future<Output = ()> + Send + 'static,
    Fn3: FnMut() -> Fut3 + Send + Sync,
    Fut3: Future<Output = ()> + Send + 'static,
    Fn4: FnMut() -> Fut4 + Send + Sync,
    Fut4: Future<Output = bool> + Send + 'static,
{
    fn on_start_game_session(
        &mut self,
        game_session: GameSession,
    ) -> Pin<Box<dyn Future<Output = ()> + Send>> {
        Box::pin((self.on_start_game_session)(game_session))
    }

    fn on_update_game_session(
        &mut self,
        update_game_session: UpdateGameSession,
    ) -> Pin<Box<dyn Future<Output = ()> + Send>> {
        Box::pin((self.on_update_game_session)(update_game_session))
    }

    fn on_process_terminate(&mut self) -> Pin<Box<dyn Future<Output = ()> + Send>> {
        Box::pin((self.on_process_terminate)())
    }

    fn on_health_check(&mut self) -> Pin<Box<dyn Future<Output = bool> + Send>> {
        Box::pin((self.on_health_check)())
    }
}
