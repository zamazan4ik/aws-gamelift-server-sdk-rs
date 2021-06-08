use crate::error::GameLiftErrorType;

const HOSTNAME: &'static str = "127.0.0.1";
const PORT: i32 = 5757;
const PID_KEY: &'static str = "pID";
const SDK_VERSION_KEY: &'static str = "sdkVersion";
const FLAVOR_KEY: &'static str = "sdkLanguage";
const FLAVOR: &'static str = "Rust";
const HEALTHCHECK_TIMEOUT_SECONDS: i32 = 60;

pub struct ServerStateInner {
    is_network_initialized: bool,
    is_connected: bool,
    process_parameters: Option<crate::process_parameters::ProcessParameters>,
    is_process_ready: bool,
    game_session_id: Option<crate::entity::GameSessionId>,
    termination_time: i64,
    http_client: crate::http_client::HttpClient,
}

impl ServerStateInner {
    pub fn new() -> Self {
        Self {
            is_network_initialized: false,
            is_connected: false,
            process_parameters: None,
            is_process_ready: false,
            game_session_id: None,
            termination_time: 0,
            http_client: crate::http_client::HttpClient::new(),
        }
    }

    fn on_start_game_session(&mut self, raw_game_session: String) {
        log::debug!(
            "ServerState got the startGameSession signal. GameSession: {}",
            raw_game_session
        );
        if !self.is_process_ready {
            log::debug!("Got a game session on inactive process. Ignoring.");
            return;
        }

        /*let game_session = <crate::protos::generated_with_pure::sdk::GameSession>::decode(
            raw_game_session.as_bytes(),
        )
        .unwrap(); //TODO: Remove unwrap
        self.game_session_id = Some(game_session.game_session_id.clone());
        (self
            .process_parameters
            .as_ref()
            .unwrap()
            .on_start_game_session)(crate::mapper::game_session_mapper(game_session));*/
    }

    fn on_terminate_process(&mut self, raw_termination_time: String) {
        log::debug!(
            "ServerState got the terminateProcess signal. TerminateProcess: {}",
            raw_termination_time
        );
        self.termination_time = raw_termination_time.parse::<i64>().unwrap(); //TODO: Remove unwrap
        (self
            .process_parameters
            .as_ref()
            .unwrap()
            .on_process_terminate)();
    }

    fn on_update_game_session(&mut self, raw_update_game_session: String) {
        log::debug!(
            "ServerState got the updateGameSession signal. UpdateGameSession: {}",
            raw_update_game_session
        );
        if !self.is_process_ready {
            log::warn!("Got an updated game session on inactive process.");
            return;
        }
        /*let update_game_session =
            <crate::protos::generated_with_pure::sdk::UpdateGameSession as prost::Message>::decode(
                raw_update_game_session.as_bytes(),
            )
            .unwrap(); //TODO: Remove unwrap
        (self
            .process_parameters
            .as_ref()
            .unwrap()
            .on_update_game_session)(crate::mapper::update_game_session_mapper(
            update_game_session,
        ));*/
    }
}

pub struct ServerState {
    inner: std::sync::Arc<std::sync::Mutex<ServerStateInner>>,
}

impl ServerState {
    pub fn new() -> Self {
        Self {
            inner: std::sync::Arc::new(std::sync::Mutex::new(ServerStateInner::new())),
        }
    }

    pub async fn initialize_networking(&mut self) -> Result<(), crate::error::GameLiftErrorType> {
        if !self.inner.lock().unwrap().is_network_initialized {}

        Ok(())
    }

    fn create_uri() -> String {
        format!("http://{}:{}", HOSTNAME, PORT)
    }

    pub async fn process_ready(
        &mut self,
        process_parameters: crate::process_parameters::ProcessParameters,
    ) -> Result<(), crate::error::GameLiftErrorType> {
        self.inner.lock().unwrap().is_process_ready = true;
        self.inner.lock().unwrap().process_parameters = Some(process_parameters);

        if !self.inner.lock().unwrap().is_network_initialized {
            return Err(crate::error::GameLiftErrorType::NetworkNotInitialized);
        }

        self.start_health_check();

        Ok(())
    }

    pub fn process_ending(&mut self) -> Result<(), crate::error::GameLiftErrorType> {
        if !self.inner.lock().unwrap().is_network_initialized {
            return Err(crate::error::GameLiftErrorType::NetworkNotInitialized);
        }
        Ok(())
    }

    pub fn activate_game_session(&mut self) -> Result<(), GameLiftErrorType> {
        if !self.inner.lock().unwrap().is_network_initialized {
            return Err(crate::error::GameLiftErrorType::NetworkNotInitialized);
        }

        if self.inner.lock().unwrap().game_session_id.is_none() {
            return Err(crate::error::GameLiftErrorType::GameSessionIdNotSet);
        }

        Ok(())
    }

    pub fn terminate_game_session(&mut self) -> Result<(), GameLiftErrorType> {
        if !self.inner.lock().unwrap().is_network_initialized {
            return Err(crate::error::GameLiftErrorType::NetworkNotInitialized);
        }

        if self.inner.lock().unwrap().game_session_id.is_none() {
            return Err(crate::error::GameLiftErrorType::GameSessionIdNotSet);
        }

        Ok(())
    }

    pub fn update_player_session_creation_policy(
        &mut self,
        player_session_policy: crate::entity::PlayerSessionCreationPolicy,
    ) -> Result<(), GameLiftErrorType> {
        if !self.inner.lock().unwrap().is_network_initialized {
            return Err(crate::error::GameLiftErrorType::NetworkNotInitialized);
        }

        if self.inner.lock().unwrap().game_session_id.is_none() {
            return Err(crate::error::GameLiftErrorType::GameSessionIdNotSet);
        }

        Ok(())
    }

    pub fn get_game_session_id(
        &mut self,
    ) -> Result<crate::entity::GameSessionId, crate::error::GameLiftErrorType> {
        match self.inner.lock().unwrap().game_session_id.as_ref() {
            Some(game_session_id) => Ok(game_session_id.clone()),
            None => Err(crate::error::GameLiftErrorType::GameSessionIdNotSet),
        }
    }

    pub fn get_termination_time(
        &mut self,
    ) -> Result<crate::entity::TerminationTimeType, crate::error::GameLiftErrorType> {
        Ok(self.inner.lock().unwrap().termination_time)
    }

    pub fn accept_player_session(
        &mut self,
        player_session_id: crate::entity::PlayerSessionId,
    ) -> Result<(), GameLiftErrorType> {
        if !self.inner.lock().unwrap().is_network_initialized {
            return Err(crate::error::GameLiftErrorType::NetworkNotInitialized);
        }

        if self.inner.lock().unwrap().game_session_id.is_none() {
            return Err(crate::error::GameLiftErrorType::GameSessionIdNotSet);
        }

        Ok(())
    }

    pub fn remove_player_session(
        &mut self,
        player_session_id: crate::entity::PlayerSessionId,
    ) -> Result<(), GameLiftErrorType> {
        if !self.inner.lock().unwrap().is_network_initialized {
            return Err(crate::error::GameLiftErrorType::NetworkNotInitialized);
        }

        if self.inner.lock().unwrap().game_session_id.is_none() {
            return Err(crate::error::GameLiftErrorType::GameSessionIdNotSet);
        }

        Ok(())
    }

    pub fn backfill_matchmaking(
        &mut self,
        request: crate::entity::StartMatchBackfillRequest,
    ) -> Result<(), GameLiftErrorType> {
        if !self.inner.lock().unwrap().is_network_initialized {
            return Err(crate::error::GameLiftErrorType::NetworkNotInitialized);
        } else {
            Ok(())
        }
    }

    pub fn stop_matchmaking(
        &mut self,
        request: crate::entity::StopMatchBackfillRequest,
    ) -> Result<(), GameLiftErrorType> {
        if !self.inner.lock().unwrap().is_network_initialized {
            return Err(crate::error::GameLiftErrorType::NetworkNotInitialized);
        } else {
            Ok(())
        }
    }

    pub fn describe_player_sessions(
        &mut self,
        request: crate::entity::DescribePlayerSessionsRequest,
    ) -> Result<(), GameLiftErrorType> {
        if !self.inner.lock().unwrap().is_network_initialized {
            return Err(crate::error::GameLiftErrorType::NetworkNotInitialized);
        } else {
            Ok(())
        }
    }

    /*public async DescribePlayerSessions(
    request: DescribePlayerSessionsRequest
    ): Promise<DescribePlayerSessionsOutcome> {
    ServerState.debug(`Describing player sessions for playerSessionId ${request.PlayerSessionId}`)
    if (!ServerState.networkInitialized) {
    return new DescribePlayerSessionsOutcome(
    new GameLiftError(GameLiftErrorType.NETWORK_NOT_INITIALIZED)
    )
    } else {
    return this.sender!.DescribePlayerSessions(request)
    }
    }*/

    pub fn shutdown(&mut self) {
        self.inner.lock().unwrap().is_network_initialized = false;
        self.inner.lock().unwrap().is_process_ready = false;
        //self.inner.lock().unwrap().
    }

    /*public Shutdown(): void {
    ServerState.networkInitialized = false
    this.network!.Disconnect()
    this.processReady = false
    }*/

    fn start_health_check(&mut self) {
        while self.inner.lock().unwrap().is_process_ready {
            self.report_health();
            // TODO: Async sleep for some time
        }
    }

    fn report_health(&mut self) {
        let health_check_result = (self
            .inner
            .lock()
            .unwrap()
            .process_parameters
            .as_ref()
            .unwrap()
            .on_health_check)();
    }
}
