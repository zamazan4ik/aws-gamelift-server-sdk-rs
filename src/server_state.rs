use rust_socketio::{Payload, SocketBuilder};

const HOSTNAME: &'static str = "127.0.0.1";
const PORT: i32 = 5757;
const PID_KEY: &'static str = "pID";
const SDK_VERSION_KEY: &'static str = "sdkVersion";
const FLAVOR_KEY: &'static str = "sdkLanguage";
const FLAVOR: &'static str = "Rust";
const HEALTHCHECK_TIMEOUT_SECONDS: i32 = 60;

pub struct ServerStateInner {
    is_network_initialized: bool,
    is_connected: std::sync::Arc<std::sync::atomic::AtomicBool>,
    sender: Option<crate::aux_proxy_message_sender::AuxProxyMessageSender>,
    network: Option<crate::network::Network>,
    process_parameters: crate::process_parameters::ProcessParameters,
    is_process_ready: bool,
    game_session_id: Option<crate::entity::GameSessionId>,
    termination_time: i64,
}

impl ServerStateInner {
    fn on_start_game_session(&mut self, raw_game_session: String) {
        log::debug!(
            "ServerState got the startGameSession signal. GameSession: {}",
            raw_game_session
        );
        if !self.is_process_ready {
            log::debug!("Got a game session on inactive process. Ignoring.");
            return;
        }

        let game_session =
            <crate::sdk::GameSession as prost::Message>::decode(raw_game_session.as_bytes())
                .unwrap(); //TODO: Remove unwrap
        self.game_session_id = Some(game_session.game_session_id.clone());
        (self.process_parameters.on_start_game_session)(crate::mapper::game_session_mapper(
            game_session,
        ));
    }

    fn on_terminate_process(&mut self, raw_termination_time: String) {
        log::debug!(
            "ServerState got the terminateProcess signal. TerminateProcess: {}",
            raw_termination_time
        );
        self.termination_time = raw_termination_time.parse::<i64>().unwrap(); //TODO: Remove unwrap
        (self.process_parameters.on_process_terminate)();
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
        let update_game_session = <crate::sdk::UpdateGameSession as prost::Message>::decode(
            raw_update_game_session.as_bytes(),
        )
        .unwrap(); //TODO: Remove unwrap
        (self.process_parameters.on_update_game_session)(
            crate::mapper::update_game_session_mapper(update_game_session),
        );
    }
}

pub struct ServerState {
    inner: std::sync::Arc<std::sync::Mutex<ServerStateInner>>,
}

impl ServerState {
    pub async fn initialize_networking(&mut self) -> Result<(), crate::error::GameLiftErrorType> {
        if !self.inner.lock().unwrap().is_network_initialized {
            let is_connected_server_state = self.inner.clone();
            let on_start_server_state = self.inner.clone();
            let on_terminate_server_state = self.inner.clone();
            let on_update_server_state = self.inner.clone();

            let socket_to_aux_proxy = SocketBuilder::new(Self::create_uri())
                .on("connect", move |_, _| {
                    log::debug!("Socket.io event triggered: connect");
                    is_connected_server_state
                        .lock()
                        .unwrap()
                        .is_connected
                        .store(true, std::sync::atomic::Ordering::SeqCst);
                })
                .on("connect_error", move |error, _| {
                    log::debug!(
                        "Socket.io event triggered: connect_error, with error: {:?}",
                        error
                    );
                })
                .on("error", move |error, _| {
                    log::debug!("Socket.io event triggered: error, with error: {:?}", error);
                })
                .on("disconnect", move |_, _| {
                    log::debug!("Socket.io event triggered: disconnect");
                })
                .on("connect_timeout", move |_, _| {
                    log::debug!("Socket.io event triggered: connect_timeout");
                })
                .on("message", move |error, _| {
                    log::debug!(
                        "Socket.io event triggered: message, with error: {:?}",
                        error
                    );
                })
                .on("StartGameSession", move |payload, _| match payload {
                    Payload::Binary(binary_payload) => {
                        log::warn!("Got StartGameSession binary payload: {:?}", binary_payload);
                    }
                    Payload::String(string_payload) => {
                        log::debug!("Got StartGameSession string payload: {:?}", string_payload);
                        on_start_server_state
                            .lock()
                            .unwrap()
                            .on_start_game_session(string_payload);
                    }
                })
                .on("TerminateProcess", move |payload, _| match payload {
                    Payload::Binary(binary_payload) => {
                        log::warn!("Got TerminateProcess binary payload: {:?}", binary_payload);
                    }
                    Payload::String(string_payload) => {
                        log::debug!("Got TerminateProcess string payload: {:?}", string_payload);
                        on_terminate_server_state
                            .lock()
                            .unwrap()
                            .on_terminate_process(string_payload);
                    }
                })
                .on("UpdateGameSession", move |payload, _| match payload {
                    Payload::Binary(binary_payload) => {
                        log::warn!("Got UpdateGameSession binary payload: {:?}", binary_payload);
                    }
                    Payload::String(string_payload) => {
                        log::debug!("Got UpdateGameSession string payload: {:?}", string_payload);
                        on_update_server_state
                            .lock()
                            .unwrap()
                            .on_update_game_session(string_payload);
                    }
                })
                .connect()
                .expect("Connection to Aux proxy failed");

            let socket_from_aux_proxy = SocketBuilder::new(Self::create_uri())
                .connect()
                .expect("Connection from Aux proxy failed");

            self.inner.lock().unwrap().sender =
                Some(crate::aux_proxy_message_sender::AuxProxyMessageSender::new(
                    socket_to_aux_proxy.clone(),
                ));
            self.inner.lock().unwrap().network = Some(crate::network::Network::new(
                socket_to_aux_proxy,
                socket_from_aux_proxy,
            ));

            //TODO: self.network.connect();

            self.inner.lock().unwrap().is_network_initialized = true;
        }

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
        self.inner.lock().unwrap().process_parameters = process_parameters;

        if !self.inner.lock().unwrap().is_network_initialized {
            return Err(crate::error::GameLiftErrorType::NetworkNotInitialized);
        }

        self.inner
            .lock()
            .unwrap()
            .sender
            .as_mut()
            .unwrap()
            .process_ready(
                self.inner.lock().unwrap().process_parameters.port,
                self.inner
                    .lock()
                    .unwrap()
                    .process_parameters
                    .log_parameters
                    .log_paths
                    .clone(),
            );

        self.start_health_check();

        Ok(())
    }

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
            .on_health_check)();
        self.inner
            .lock()
            .unwrap()
            .sender
            .as_mut()
            .unwrap()
            .report_health(health_check_result);
    }
}
