use std::{
    sync::{
        atomic::{AtomicBool, Ordering},
        Arc,
    },
    time::{Duration, SystemTime},
};

use crate::{
    error::GameLiftErrorType,
    model::{self, request, responce_result},
    process_parameters::ProcessParameters,
    server_parameters::ServerParameters,
    web_socket_listener::{GameLiftEventInner, WebSocketListener},
};
use tokio::sync::mpsc;

const ENVIRONMENT_VARIABLE_WEBSOCKET_URL: &str = "GAMELIFT_SDK_WEBSOCKET_URL";
const ENVIRONMENT_VARIABLE_PROCESS_ID: &str = "GAMELIFT_SDK_PROCESS_ID";
const ENVIRONMENT_VARIABLE_HOST_ID: &str = "GAMELIFT_SDK_HOST_ID";
const ENVIRONMENT_VARIABLE_FLEET_ID: &str = "GAMELIFT_SDK_FLEET_ID";
const ENVIRONMENT_VARIABLE_AUTH_TOKEN: &str = "GAMELIFT_SDK_AUTH_TOKEN";

const HEALTHCHECK_INTERVAL_SECONDS: u64 = 60;
const HEALTHCHECK_MAX_JITTER_SECONDS: u64 = 10;
const HEALTHCHECK_TIMEOUT_SECONDS: u64 =
    HEALTHCHECK_INTERVAL_SECONDS - HEALTHCHECK_MAX_JITTER_SECONDS;
const SDK_LANGUAGE: &str = "Rust";

struct ServerStateInner {
    is_process_ready: AtomicBool,
    game_session_id: parking_lot::Mutex<Option<String>>,
    termination_time: parking_lot::Mutex<Option<SystemTime>>,
    websocket_listener: tokio::sync::RwLock<WebSocketListener>,
    fleet_id: String,
    host_id: String,
    process_id: String,
}

impl ServerStateInner {
    async fn new(server_parameters: ServerParameters) -> Result<Arc<Self>, GameLiftErrorType> {
        let websocket_listener = WebSocketListener::connect(&server_parameters).await?;
        let this = Arc::new(Self {
            is_process_ready: AtomicBool::new(false),
            game_session_id: parking_lot::Mutex::new(None),
            termination_time: parking_lot::Mutex::new(None),
            websocket_listener: tokio::sync::RwLock::new(websocket_listener),
            fleet_id: server_parameters.fleet_id,
            host_id: server_parameters.host_id,
            process_id: server_parameters.process_id,
        });
        Ok(this)
    }

    fn is_process_ready(&self) -> bool {
        self.is_process_ready.load(Ordering::Relaxed)
    }

    fn set_is_process_ready(&self, value: bool) {
        self.is_process_ready.store(value, Ordering::Relaxed);
    }

    fn get_game_session_id(&self) -> Option<String> {
        self.game_session_id.lock().clone()
    }

    fn get_termination_time(&self) -> Option<SystemTime> {
        *self.termination_time.lock()
    }

    async fn run_event_listener(
        self: Arc<Self>,
        mut event_receiver: mpsc::Receiver<GameLiftEventInner>,
        process_parameters: ProcessParameters,
    ) {
        let mut interval = tokio::time::interval(Duration::from_secs(HEALTHCHECK_TIMEOUT_SECONDS));
        log::debug!("Health check and event listening started.");

        loop {
            let event = tokio::select! {
                e = event_receiver.recv() => e,
                _ = interval.tick() => Some(GameLiftEventInner::OnHealthCheck()),
            };

            let event = match event {
                Some(e) => e,
                None => break,
            };

            match event {
                GameLiftEventInner::OnStartGameSession(msg) => {
                    self.on_start_game_session(&process_parameters, msg).await;
                }
                GameLiftEventInner::OnUpdateGameSession(msg) => {
                    self.on_update_game_session(&process_parameters, msg).await;
                }
                GameLiftEventInner::OnTerminateProcess(msg) => {
                    self.on_terminate_process(&process_parameters, msg.termination_time).await;
                }
                GameLiftEventInner::OnRefreshConnection(msg) => {
                    log::info!("Refresh connection");

                    let server_parameters = ServerParameters {
                        web_socket_url: msg.refresh_connection_endpoint,
                        process_id: self.process_id.clone(),
                        host_id: self.host_id.clone(),
                        fleet_id: self.fleet_id.clone(),
                        auth_token: msg.auth_token,
                    };

                    // Reserves locks to prevent new requests from being made
                    let mut lock = self.websocket_listener.write().await;
                    match WebSocketListener::connect(&server_parameters).await {
                        Ok(mut websocket_listener) => {
                            event_receiver = websocket_listener
                                .take_event_receiver()
                                .expect("Need to continue listening");
                            *lock = websocket_listener;
                        }
                        Err(error) => {
                            log::error!("Refresh connection failure: {error}");
                            break;
                        }
                    };
                }
                GameLiftEventInner::OnHealthCheck() => {
                    self.report_health(&process_parameters).await;
                }
            }
        }

        log::debug!("Health check and event listening ended.");
    }

    async fn on_start_game_session(
        &self,
        process_parameters: &ProcessParameters,
        mut game_session: model::GameSession,
    ) {
        // Inject data that already exists on the server
        game_session.fleet_id = self.fleet_id.clone();

        if !self.is_process_ready() {
            log::debug!("Got a game session on inactive process. Ignoring.");
            return;
        }

        *self.game_session_id.lock() = Some(game_session.game_session_id.clone());
        (process_parameters.on_start_game_session)(game_session).await;
    }

    async fn on_terminate_process(
        &self,
        process_parameters: &ProcessParameters,
        termination_time: SystemTime,
    ) {
        log::debug!(
            "ServerState got the terminateProcess signal. TerminateProcess: {:?}",
            termination_time
        );

        *self.termination_time.lock() = Some(termination_time);
        (process_parameters.on_process_terminate)().await;
    }

    async fn on_update_game_session(
        &self,
        process_parameters: &ProcessParameters,
        update_game_session: model::UpdateGameSession,
    ) {
        if !self.is_process_ready() {
            log::warn!("Got an updated game session on inactive process.");
            return;
        }

        (process_parameters.on_update_game_session)(update_game_session).await;
    }

    async fn report_health(&self, process_parameters: &ProcessParameters) {
        if !self.is_process_ready() {
            log::debug!("Reporting Health on an inactive process. Ignoring.");
            return;
        }

        log::debug!("Reporting health using the OnHealthCheck callback.");

        let callback = (process_parameters.on_health_check)();
        let result = tokio::time::timeout(
            std::time::Duration::from_secs(HEALTHCHECK_TIMEOUT_SECONDS),
            callback,
        )
        .await;

        let health_status = result.unwrap_or(false);
        let msg = request::HeartbeatServerProcessRequest { health_status };
        if let Err(error) = self.request(msg).await {
            log::warn!("Could not send health starus: {:?}", error);
        }
    }

    pub async fn request<T>(
        &self,
        request: T,
    ) -> Result<<T as model::protocol::RequestContent>::Response, GameLiftErrorType>
    where
        T: model::protocol::RequestContent,
    {
        let lock = self.websocket_listener.read().await;
        lock.request(request).await
    }
}

pub struct ServerState {
    inner: Arc<ServerStateInner>,
}

impl ServerState {
    pub async fn process_ready(
        &self,
        process_parameters: ProcessParameters,
    ) -> Result<(), GameLiftErrorType> {
        let inner = &self.inner;

        let event_receiver = {
            let mut lock = inner.websocket_listener.write().await;
            lock.take_event_receiver().expect("process_ready() can only be called once")
        };

        inner.set_is_process_ready(true);

        let msg = request::ActivateServerProcessRequest {
            sdk_version: crate::api::Api::get_sdk_version().to_owned(),
            sdk_language: SDK_LANGUAGE.to_owned(),
            port: process_parameters.port,
            log_paths: process_parameters.log_parameters.log_paths.clone(),
        };
        let result = self.inner.request(msg).await;

        tokio::spawn(inner.clone().run_event_listener(event_receiver, process_parameters));

        result
    }

    pub async fn process_ending(&self) -> Result<(), GameLiftErrorType> {
        self.inner.set_is_process_ready(false);

        let msg = request::TerminateServerProcessRequest {};
        self.inner.request(msg).await
    }

    pub async fn activate_game_session(&self) -> Result<(), GameLiftErrorType> {
        let game_session_id = self.inner.get_game_session_id();
        if let Some(game_session_id) = game_session_id {
            let msg = request::ActivateGameSessionRequest { game_session_id };
            self.inner.request(msg).await
        } else {
            Err(GameLiftErrorType::GameSessionIdNotSet)
        }
    }

    pub async fn get_game_session_id(&self) -> Result<String, GameLiftErrorType> {
        match self.inner.get_game_session_id() {
            Some(game_session_id) => Ok(game_session_id),
            None => Err(GameLiftErrorType::GameSessionIdNotSet),
        }
    }

    pub async fn get_termination_time(&self) -> Result<SystemTime, GameLiftErrorType> {
        match self.inner.get_termination_time() {
            Some(value) => Ok(value),
            None => Err(GameLiftErrorType::TerminationTimeNotSet),
        }
    }

    pub async fn update_player_session_creation_policy(
        &self,
        player_session_policy: model::PlayerSessionCreationPolicy,
    ) -> Result<(), GameLiftErrorType> {
        let inner = &self.inner;

        let game_session_id = inner.get_game_session_id();
        if let Some(game_session_id) = game_session_id {
            let msg = request::UpdatePlayerSessionCreationPolicyRequest {
                game_session_id,
                player_session_policy,
            };
            self.inner.request(msg).await
        } else {
            Err(GameLiftErrorType::GameSessionIdNotSet)
        }
    }

    pub async fn accept_player_session(
        &self,
        player_session_id: impl Into<String>,
    ) -> Result<(), GameLiftErrorType> {
        let inner = &self.inner;

        let game_session_id = inner.get_game_session_id();
        let player_session_id = player_session_id.into();
        if let Some(game_session_id) = game_session_id {
            let msg = request::AcceptPlayerSessionRequest { game_session_id, player_session_id };
            self.inner.request(msg).await
        } else {
            Err(GameLiftErrorType::GameSessionIdNotSet)
        }
    }

    pub async fn remove_player_session(
        &self,
        player_session_id: impl Into<String>,
    ) -> Result<(), GameLiftErrorType> {
        let inner = &self.inner;

        let game_session_id = inner.get_game_session_id();
        let player_session_id = player_session_id.into();
        if let Some(game_session_id) = game_session_id {
            let msg = request::RemovePlayerSessionRequest { game_session_id, player_session_id };
            self.inner.request(msg).await
        } else {
            Err(GameLiftErrorType::GameSessionIdNotSet)
        }
    }

    pub async fn describe_player_sessions(
        &self,
        request: request::DescribePlayerSessionsRequest,
    ) -> Result<responce_result::DescribePlayerSessionsResult, GameLiftErrorType> {
        self.inner.request(request).await
    }

    pub async fn backfill_matchmaking(
        &self,
        request: request::StartMatchBackfillRequest,
    ) -> Result<responce_result::StartMatchBackfillResult, GameLiftErrorType> {
        self.inner.request(request).await
    }

    pub async fn stop_matchmaking(
        &self,
        request: request::StopMatchBackfillRequest,
    ) -> Result<(), GameLiftErrorType> {
        self.inner.request(request).await
    }

    pub async fn initialize_networking(
        server_parameters: ServerParameters,
    ) -> Result<Self, GameLiftErrorType> {
        let server_parameters = ServerParameters {
            web_socket_url: std::env::var(ENVIRONMENT_VARIABLE_WEBSOCKET_URL)
                .unwrap_or(server_parameters.web_socket_url),
            process_id: std::env::var(ENVIRONMENT_VARIABLE_PROCESS_ID)
                .unwrap_or(server_parameters.process_id),
            host_id: std::env::var(ENVIRONMENT_VARIABLE_HOST_ID)
                .unwrap_or(server_parameters.host_id),
            fleet_id: std::env::var(ENVIRONMENT_VARIABLE_FLEET_ID)
                .unwrap_or(server_parameters.fleet_id),
            auth_token: std::env::var(ENVIRONMENT_VARIABLE_AUTH_TOKEN)
                .unwrap_or(server_parameters.auth_token),
        };

        Ok(Self { inner: ServerStateInner::new(server_parameters).await? })
    }

    pub async fn get_compute_certificate(
        &self,
    ) -> Result<responce_result::GetComputeCertificateResult, GameLiftErrorType> {
        let msg = request::GetComputeCertificateRequest {};
        self.inner.request(msg).await
    }

    pub async fn get_fleet_role_credentials(
        &self,
        request: model::GetFleetRoleCredentialsRequest,
    ) -> Result<responce_result::GetFleetRoleCredentialsResult, GameLiftErrorType> {
        self.inner.request(request).await
    }

    pub async fn request<T>(
        &self,
        request: T,
    ) -> Result<<T as model::protocol::RequestContent>::Response, GameLiftErrorType>
    where
        T: model::protocol::RequestContent,
    {
        self.inner.request(request).await
    }
}
