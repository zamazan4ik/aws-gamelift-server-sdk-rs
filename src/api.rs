use crate::{
    error::GameLiftErrorType,
    model::{request, responce_result},
};

const SDK_VERSION: &str = "5.0.0";

pub struct Api {
    state: crate::server_state::ServerState,
}

impl Api {
    /// Returns the current version number of the SDK built into the server
    /// process.
    pub fn get_sdk_version() -> &'static str {
        SDK_VERSION
    }

    /// Initializes the GameLift SDK. This method should be called on launch,
    /// before any other GameLift-related initialization occurs.
    pub async fn init_sdk(
        server_parameters: crate::server_parameters::ServerParameters,
    ) -> Result<Self, GameLiftErrorType> {
        let state =
            crate::server_state::ServerState::initialize_networking(server_parameters).await?;
        Ok(Self { state })
    }

    /// Notifies the GameLift service that the server process is ready to host
    /// game sessions. Call this method after successfully invoking
    /// [init_sdk](crate::api::Api::init_sdk) and completing setup tasks
    /// that are required before the server process can host a game session.
    /// This method should be called only once per process.
    pub async fn process_ready(
        &mut self,
        process_parameters: crate::process_parameters::ProcessParameters,
    ) -> Result<(), GameLiftErrorType> {
        self.state.process_ready(process_parameters).await
    }

    /// Notifies the GameLift service that the server process is shutting down.
    /// This method should be called after all other cleanup tasks, including
    /// shutting down all active game sessions. This method should exit with an
    /// exit code of 0; a non-zero exit code results in an event message that
    /// the process did not exit cleanly.
    pub async fn process_ending(&self) -> Result<(), GameLiftErrorType> {
        self.state.process_ending().await
    }

    /// Notifies the GameLift service that the server process has activated a
    /// game session and is now ready to receive player connections. This action
    /// should be called as part of the on_start_game_session() callback
    /// function, after all game session initialization has been completed.
    pub async fn activate_game_session(&self) -> Result<(), GameLiftErrorType> {
        self.state.activate_game_session().await
    }

    /// Updates the current game session's ability to accept new player
    /// sessions. A game session can be set to either accept or deny all new
    /// player sessions. (See also the update_game_session() action in the
    /// GameLift Service API Reference).
    pub async fn update_player_session_creation_policy(
        &self,
        player_session_policy: crate::entity::PlayerSessionCreationPolicy,
    ) -> Result<(), GameLiftErrorType> {
        self.state.update_player_session_creation_policy(player_session_policy).await
    }

    /// Retrieves the ID of the game session currently being hosted by the
    /// server process, if the server process is active.
    pub async fn get_game_session_id(
        &self,
    ) -> Result<crate::entity::GameSessionId, crate::error::GameLiftErrorType> {
        self.state.get_game_session_id().await
    }

    /// Returns the time that a server process is scheduled to be shut down, if
    /// a termination time is available. A server process takes this action
    /// after receiving an on_process_terminate() callback from the GameLift
    /// service. GameLift may call on_process_terminate() for the following
    /// reasons: (1) for poor health (the server process has reported port
    /// health or has not responded to GameLift, (2) when terminating the
    /// instance during a scale-down event, or (3) when an instance is being
    /// terminated due to a spot-instance interruption.
    ///
    /// If the process has received an on_process_terminate() callback, the
    /// value returned is the estimated termination time. If the process has
    /// not received an on_process_terminate() callback, an error message is
    /// returned. Learn more about shutting down a server process.
    pub async fn get_termination_time(
        &self,
    ) -> Result<crate::entity::TerminationTimeType, crate::error::GameLiftErrorType> {
        self.state.get_termination_time().await
    }

    /// Notifies the GameLift service that a player with the specified player
    /// session ID has connected to the server process and needs validation.
    /// GameLift verifies that the player session ID is valid—that is, that the
    /// player ID has reserved a player slot in the game session. Once
    /// validated, GameLift changes the status of the player slot from RESERVED
    /// to ACTIVE.
    pub async fn accept_player_session(
        &self,
        player_session_id: crate::entity::PlayerSessionId,
    ) -> Result<(), GameLiftErrorType> {
        self.state.accept_player_session(player_session_id).await
    }

    /// Notifies the GameLift service that a player with the specified player
    /// session ID has disconnected from the server process. In response,
    /// GameLift changes the player slot to available, which allows it to be
    /// assigned to a new player.
    pub async fn remove_player_session(
        &self,
        player_session_id: crate::entity::PlayerSessionId,
    ) -> Result<(), GameLiftErrorType> {
        self.state.remove_player_session(player_session_id).await
    }

    /// Retrieves player session data, including settings, session metadata, and
    /// player data. Use this action to get information for a single player
    /// session, for all player sessions in a game session, or for all player
    /// sessions associated with a single player ID.
    pub async fn describe_player_sessions(
        &self,
        describe_player_sessions_request: request::DescribePlayerSessionsRequest,
    ) -> Result<responce_result::DescribePlayerSessionsResult, GameLiftErrorType> {
        self.state.describe_player_sessions(describe_player_sessions_request).await
    }

    /// Sends a request to find new players for open slots in a game session
    /// created with FlexMatch. See also the AWS SDK action
    /// [start_match_backfill](crate::api::Api::start_match_backfill). With this
    /// action, match backfill requests can be initiated by a game server
    /// process that is hosting the game session. Learn more about the
    /// FlexMatch backfill feature.
    ///
    /// This action is asynchronous. If new players are successfully matched,
    /// the GameLift service delivers updated matchmaker data using the callback
    /// function on_update_game_session().
    ///
    /// A server process can have only one active match backfill request at a
    /// time. To send a new request, first call
    /// [stop_match_backfill](crate::api::Api::stop_match_backfill) to cancel
    /// the original request.
    pub async fn start_match_backfill(
        &self,
        request: request::StartMatchBackfillRequest,
    ) -> Result<responce_result::StartMatchBackfillResult, GameLiftErrorType> {
        self.state.backfill_matchmaking(request).await
    }

    /// Cancels an active match backfill request that was created with
    /// [start_match_backfill](crate::api::Api::start_match_backfill). See also
    /// the AWS SDK action StopMatchmaking(). Learn more about the FlexMatch
    /// backfill feature.
    pub async fn stop_match_backfill(
        &self,
        request: request::StopMatchBackfillRequest,
    ) -> Result<(), GameLiftErrorType> {
        self.state.stop_matchmaking(request).await
    }

    /// Retrieves the file location of a pem-encoded TLS certificate that is
    /// associated with the fleet and its instances. This certificate is
    /// generated when a new fleet is created with the certificate configuration
    /// set to GENERATED. Use this certificate to establish a secure connection
    /// with a game client and to encrypt client/server communication.
    pub async fn get_instance_certificate(
        &self,
    ) -> Result<responce_result::GetComputeCertificateResult, GameLiftErrorType> {
        self.state.get_instance_certificate().await
    }
}
