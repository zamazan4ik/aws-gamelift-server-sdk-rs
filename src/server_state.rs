use futures_util::stream::StreamExt;
use rust_socketio::{Payload, Socket, SocketBuilder};
use tokio_tungstenite::connect_async;

const HOSTNAME: &'static str = "127.0.0.1";
const PORT: i32 = 5757;
const PID_KEY: &'static str = "pID";
const SDK_VERSION_KEY: &'static str = "sdkVersion";
const FLAVOR_KEY: &'static str = "sdkLanguage";
const FLAVOR: &'static str = "Rust";
const HEALTHCHECK_TIMEOUT_SECONDS: i32 = 60;

pub struct ServerState {
    is_network_initialized: bool,
    is_connected: std::sync::Arc<std::sync::atomic::AtomicBool>,
    sender: Option<crate::aux_proxy_message_sender::AuxProxyMessageSender>,
    network: Option<crate::network::Network>,
    process_parameters: crate::process_parameters::ProcessParameters,
    is_process_ready: bool,
    game_session_id: Option<crate::entity::GameSessionId>,
    termination_time: i64,
}

impl ServerState {
    pub async fn initialize_networking(&mut self) -> Result<(), crate::error::GameLiftErrorType> {
        if !self.is_network_initialized {
            let callback = |payload: Payload, mut socket: Socket| {
                match payload {
                    Payload::String(str) => println!("Received: {}", str),
                    Payload::Binary(bin_data) => println!("Received bytes: {:#?}", bin_data),
                }
                socket.emit("test", "test").expect("Server unreachable")
            };

            let mut is_connected = self.is_connected.clone();
            let mut socket_to_aux_proxy = SocketBuilder::new(Self::create_uri())
                .on("connect", move |_, _| {
                    log::debug!("Socket.io event triggered: connect");
                    (*is_connected).store(true, std::sync::atomic::Ordering::SeqCst);
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
                .on("StartGameSession", move |_, _| {
                    //self.sender.unwrap().
                })
                .on("TerminateProcess", move |_, _| {})
                .on("UpdateGameSession", move |_, _| {})
                .connect()
                .expect("Connection to Aux proxy failed");

            let mut socket_from_aux_proxy = SocketBuilder::new(Self::create_uri())
                .connect()
                .expect("Connection from Aux proxy failed");

            self.sender = Some(crate::aux_proxy_message_sender::AuxProxyMessageSender::new(
                socket_to_aux_proxy.clone(),
            ));
            self.network = Some(crate::network::Network::new(
                socket_to_aux_proxy,
                socket_from_aux_proxy,
            ));

            //TODO: self.network.connect();

            self.is_network_initialized = true;
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
        self.is_process_ready = true;
        self.process_parameters = process_parameters;

        if !self.is_network_initialized {
            return Err(crate::error::GameLiftErrorType::NetworkNotInitialized);
        }

        self.sender.as_mut().unwrap().process_ready(
            self.process_parameters.port,
            self.process_parameters.log_parameters.log_paths.clone(),
        );

        self.start_health_check();

        Ok(())
    }

    fn start_health_check(&mut self) {
        while self.is_process_ready {
            self.report_health();
            // TODO: Async sleep for some time
        }
    }

    fn report_health(&mut self) {
        let health_check_result = (self.process_parameters.on_health_check)();
        self.sender
            .as_mut()
            .unwrap()
            .report_health(health_check_result);
    }

    fn on_start_game_session(&mut self, raw_game_session: String) {
        log::debug!("ServerState got the startGameSession signal");
        if !self.is_process_ready {
            log::debug!("Got a game session on inactive process. Sending false ack.");
            // ack(false)
            return;
        }

        log::debug!("Sending true ack.");
        // ack(true);
        let mut game_session =
            <crate::sdk::GameSession as prost::Message>::decode(raw_game_session.as_bytes())
                .unwrap(); //TODO: Remove unwrap
        self.game_session_id = Some(game_session.game_session_id.clone());
        (self.process_parameters.on_start_game_session)(crate::mapper::game_session_mapper(
            game_session,
        ));
    }
}
