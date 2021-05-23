const sdk_version: &'static str = "4.0.2";

struct Api {
    state: crate::server_state::ServerState,
}

impl Api {
    pub fn new() -> Self {
        Self {
            state: crate::server_state::ServerState::new(),
        }
    }

    pub fn get_sdk_version() -> &'static str {
        sdk_version
    }

    pub fn init_sdk(&mut self) {
        self.state.initialize_networking();
    }

    pub fn process_ready(
        &mut self,
        process_parameters: crate::process_parameters::ProcessParameters,
    ) {
        self.state.process_ready(process_parameters);
    }

    pub fn process_ending(&mut self) {
        self.state.process_ending();
    }

    pub fn activate_game_session(&mut self) {
        self.state.activate_game_session();
    }

    pub fn terminate_game_session(&mut self) {
        self.state.terminate_game_session();
    }

    pub fn update_player_session_creation_policy(
        &mut self,
        player_session_policy: crate::entity::PlayerSessionCreationPolicy,
    ) {
        self.state
            .update_player_session_creation_policy(player_session_policy);
    }

    pub fn get_game_session_id(
        &mut self,
    ) -> Result<crate::entity::GameSessionId, crate::error::GameLiftErrorType> {
        self.state.get_game_session_id()
    }

    pub fn get_termination_time(
        &mut self,
    ) -> Result<crate::entity::TerminationTimeType, crate::error::GameLiftErrorType> {
        self.state.get_termination_time()
    }

    pub fn accept_player_session(&mut self, player_session_id: crate::entity::PlayerSessionId) {
        self.state.accept_player_session(player_session_id);
    }

    pub fn remove_player_session(&mut self, player_session_id: crate::entity::PlayerSessionId) {
        self.state.remove_player_session(player_session_id);
    }

    pub fn describe_player_sessions(
        &mut self,
        describe_player_sessions_request: crate::entity::DescribePlayerSessionsRequest,
    ) {
    }

    pub fn stop_match_backfill_request(
        &mut self,
        request: crate::entity::StopMatchBackfillRequest,
    ) {
        self.state.stop_matchmaking(request);
    }
}

/*
public static DescribePlayerSessions(
describePlayerSessionsRequest: DescribePlayerSessionsRequest
): Promise<DescribePlayerSessionsOutcome> {
return ServerState.Instance.DescribePlayerSessions(describePlayerSessionsRequest)
}

public static StartMatchBackfill(request: StartMatchBackfillRequest): Promise<StartMatchBackfillOutcome> {
return ServerState.Instance.BackfillMatchmaking(request)
}

public static StopMatchBackfill(request: StopMatchBackfillRequest): Promise<GenericOutcome> {
return ServerState.Instance.StopMatchmaking(request)
}

public static Destroy(): GenericOutcome {
ServerState.Instance.Shutdown()
return new GenericOutcome()
}*/
