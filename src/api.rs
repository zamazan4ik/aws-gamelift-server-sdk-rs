use crate::entity::{
    DescribePlayerSessionsResult, GetInstanceCertificateResult, StartMatchBackfillResult,
};
use crate::error::GameLiftErrorType;

pub const SDK_VERSION: &'static str = "4.0.2";

pub struct Api {
    state: crate::server_state::ServerState,
}

impl Api {
    pub fn new() -> Self {
        Self {
            state: crate::server_state::ServerState::new(),
        }
    }

    pub fn get_sdk_version() -> &'static str {
        SDK_VERSION
    }

    pub async fn init_sdk(&mut self) -> Result<(), GameLiftErrorType> {
        self.state.initialize_networking().await
    }

    pub async fn process_ready(
        &mut self,
        process_parameters: crate::process_parameters::ProcessParameters,
    ) -> Result<(), GameLiftErrorType> {
        self.state.process_ready(process_parameters).await
    }

    pub async fn process_ending(&mut self) -> Result<(), GameLiftErrorType> {
        self.state.process_ending().await
    }

    pub async fn activate_game_session(&mut self) -> Result<(), GameLiftErrorType> {
        self.state.activate_game_session().await
    }

    pub async fn terminate_game_session(&mut self) -> Result<(), GameLiftErrorType> {
        self.state.terminate_game_session().await
    }

    pub async fn update_player_session_creation_policy(
        &mut self,
        player_session_policy: crate::entity::PlayerSessionCreationPolicy,
    ) -> Result<(), GameLiftErrorType> {
        self.state
            .update_player_session_creation_policy(player_session_policy)
            .await
    }

    pub async fn get_game_session_id(
        &mut self,
    ) -> Result<crate::entity::GameSessionId, crate::error::GameLiftErrorType> {
        self.state.get_game_session_id().await
    }

    pub async fn get_termination_time(
        &mut self,
    ) -> Result<crate::entity::TerminationTimeType, crate::error::GameLiftErrorType> {
        self.state.get_termination_time().await
    }

    pub async fn accept_player_session(
        &mut self,
        player_session_id: crate::entity::PlayerSessionId,
    ) -> Result<(), GameLiftErrorType> {
        self.state.accept_player_session(player_session_id).await
    }

    pub async fn remove_player_session(
        &mut self,
        player_session_id: crate::entity::PlayerSessionId,
    ) -> Result<(), GameLiftErrorType> {
        self.state.remove_player_session(player_session_id).await
    }

    pub async fn describe_player_sessions(
        &mut self,
        describe_player_sessions_request: crate::entity::DescribePlayerSessionsRequest,
    ) -> Result<DescribePlayerSessionsResult, GameLiftErrorType> {
        self.state
            .describe_player_sessions(describe_player_sessions_request)
            .await
    }

    pub async fn start_match_backfill(
        &mut self,
        request: crate::entity::StartMatchBackfillRequest,
    ) -> Result<StartMatchBackfillResult, GameLiftErrorType> {
        self.state.backfill_matchmaking(request).await
    }

    pub async fn stop_match_backfill(
        &mut self,
        request: crate::entity::StopMatchBackfillRequest,
    ) -> Result<(), GameLiftErrorType> {
        self.state.stop_matchmaking(request).await
    }

    pub async fn get_instance_certificate(
        &self,
    ) -> Result<GetInstanceCertificateResult, GameLiftErrorType> {
        self.state.get_instance_certificate().await
    }

    pub async fn destroy(&self) -> bool {
        self.state.shutdown().await
    }
}
