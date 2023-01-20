use std::sync::{
    atomic::{AtomicBool, Ordering},
    Arc,
};

use crate::{
    error::GameLiftErrorType,
    process_parameters::ProcessParameters,
};
use tokio::task::JoinHandle;

const ENVIRONMENT_VARIABLE_WEBSOCKET_URL: &str = "GAMELIFT_SDK_WEBSOCKET_URL";
const ENVIRONMENT_VARIABLE_PROCESS_ID: &str = "GAMELIFT_SDK_PROCESS_ID";
const ENVIRONMENT_VARIABLE_HOST_ID: &str = "GAMELIFT_SDK_HOST_ID";
const ENVIRONMENT_VARIABLE_FLEET_ID: &str = "GAMELIFT_SDK_FLEET_ID";
const ENVIRONMENT_VARIABLE_AUTH_TOKEN: &str = "GAMELIFT_SDK_AUTH_TOKEN";

const HEALTHCHECK_TIMEOUT_SECONDS: u64 = 60;

#[derive(Default)]
pub struct ServerStateInner {
    is_process_ready: AtomicBool,
    game_session_id: parking_lot::Mutex<Option<crate::entity::GameSessionId>>,
    termination_time: parking_lot::Mutex<Option<crate::entity::TerminationTimeType>>,
    process_parameters: parking_lot::Mutex<Option<crate::process_parameters::ProcessParameters>>,
}

impl ServerStateInner {
    fn is_process_ready(&self) -> bool {
        self.is_process_ready.load(Ordering::Relaxed)
    }

    fn set_is_process_ready(&self, value: bool) {
        self.is_process_ready.store(value, Ordering::Relaxed);
    }

    fn get_game_session_id(&self) -> Option<crate::entity::GameSessionId> {
        self.game_session_id.lock().clone()
    }

    fn get_termination_time(&self) -> Option<crate::entity::TerminationTimeType> {
        *self.termination_time.lock()
    }

    async fn on_start_game_session(&self, mut game_session: crate::entity::GameSession) {
        if !self.is_process_ready() {
            log::debug!("Got a game session on inactive process. Ignoring.");
            return;
        }

        *self.game_session_id.lock() = Some(game_session.game_session_id.clone().unwrap());
        let callback = {
            let lock = self.process_parameters.lock();
            (lock.as_ref().unwrap().on_start_game_session)(game_session)
        };
        callback.await;
    }

    async fn on_terminate_process(&self, termination_time: i64) {
        log::debug!(
            "ServerState got the terminateProcess signal. TerminateProcess: {}",
            termination_time
        );
        *self.termination_time.lock() = Some(termination_time);
        let callback = {
            let lock = self.process_parameters.lock();
            (lock.as_ref().unwrap().on_process_terminate)()
        };
        callback.await;
    }

    async fn on_update_game_session(
        &self,
        game_session: crate::entity::GameSession,
        update_reason: crate::entity::UpdateReason,
        backfill_ticket_id: String,
    ) {
        if !self.is_process_ready() {
            log::warn!("Got an updated game session on inactive process.");
            return;
        }
        let callback = {
            let lock = self.process_parameters.lock();
            (lock.as_ref().unwrap().on_update_game_session)(crate::entity::UpdateGameSession {
                game_session: Some(game_session),
                update_reason,
                backfill_ticket_id,
            })
        };
        callback.await;
    }
}

pub struct ServerState {
    inner: Arc<ServerStateInner>,
    http_client: Arc<crate::http_client::HttpClient>,
    websocket_listener: Option<crate::web_socket_listener::WebSocketListener>,
    health_report_task: Option<JoinHandle<()>>,
    fleet_id: String,
    host_id: String,
    process_id: String,
}

impl Default for ServerState {
    fn default() -> Self {
        Self {
            inner: Arc::new(ServerStateInner::default()),
            http_client: Arc::new(crate::http_client::HttpClient::default()),
            websocket_listener: None,
            health_report_task: None,
            fleet_id: String::default(),
            host_id: String::default(),
            process_id: String::default(),
        }
    }
}

impl ServerState {
    pub async fn process_ready(
        &mut self,
        process_parameters: ProcessParameters,
    ) -> Result<(), GameLiftErrorType> {
        let port = process_parameters.port;
        let log_paths = process_parameters.log_parameters.log_paths.clone();

        let result = {
            let inner = &self.inner;

            inner.set_is_process_ready(true);
            *inner.process_parameters.lock() = Some(process_parameters);

            self.http_client.process_ready(port, log_paths).await
        };

        self.start_health_check().await;

        result
    }

    pub async fn process_ending(&self) -> Result<(), crate::error::GameLiftErrorType> {
        let inner = &self.inner;

        inner.set_is_process_ready(false);
        self.http_client.process_ending().await
    }

    pub async fn activate_game_session(&self) -> Result<(), GameLiftErrorType> {
        let game_session_id = self.inner.get_game_session_id();
        if let Some(game_session_id) = game_session_id {
            self.http_client.activate_game_session(game_session_id).await
        } else {
            Err(crate::error::GameLiftErrorType::GameSessionIdNotSet)
        }
    }

    pub async fn terminate_game_session(&self) -> Result<(), GameLiftErrorType> {
        let inner = &self.inner;

        let game_session_id = inner.get_game_session_id();
        if let Some(game_session_id) = game_session_id {
            self.http_client.terminate_game_session(game_session_id).await
        } else {
            Err(crate::error::GameLiftErrorType::GameSessionIdNotSet)
        }
    }

    pub async fn get_game_session_id(
        &self,
    ) -> Result<crate::entity::GameSessionId, GameLiftErrorType> {
        match self.inner.get_game_session_id() {
            Some(game_session_id) => Ok(game_session_id),
            None => Err(GameLiftErrorType::GameSessionIdNotSet),
        }
    }

    pub async fn get_termination_time(
        &self,
    ) -> Result<crate::entity::TerminationTimeType, GameLiftErrorType> {
        match self.inner.get_termination_time() {
            Some(value) => Ok(value),
            None => Err(GameLiftErrorType::TerminationTimeNotSet),
        }
    }

    pub async fn update_player_session_creation_policy(
        &self,
        player_session_policy: crate::entity::PlayerSessionCreationPolicy,
    ) -> Result<(), GameLiftErrorType> {
        let inner = &self.inner;

        let game_session_id = inner.get_game_session_id();
        if let Some(game_session_id) = game_session_id {
            self.http_client
                .update_player_session_creation_policy(game_session_id, player_session_policy)
                .await
        } else {
            Err(GameLiftErrorType::GameSessionIdNotSet)
        }
    }

    pub async fn accept_player_session(
        &self,
        player_session_id: crate::entity::PlayerSessionId,
    ) -> Result<(), GameLiftErrorType> {
        let inner = &self.inner;

        let game_session_id = inner.get_game_session_id();
        if let Some(game_session_id) = game_session_id {
            self.http_client.accept_player_session(player_session_id, game_session_id).await
        } else {
            Err(GameLiftErrorType::GameSessionIdNotSet)
        }
    }

    pub async fn remove_player_session(
        &self,
        player_session_id: crate::entity::PlayerSessionId,
    ) -> Result<(), GameLiftErrorType> {
        let inner = &self.inner;

        let game_session_id = inner.get_game_session_id();
        if let Some(game_session_id) = game_session_id {
            self.http_client.remove_player_session(player_session_id, game_session_id).await
        } else {
            Err(GameLiftErrorType::GameSessionIdNotSet)
        }
    }

    pub async fn describe_player_sessions(
        &self,
        request: crate::entity::DescribePlayerSessionsRequest,
    ) -> Result<crate::entity::DescribePlayerSessionsResult, GameLiftErrorType> {
        self.http_client.describe_player_sessions(request).await
    }

    pub async fn backfill_matchmaking(
        &self,
        request: crate::entity::StartMatchBackfillRequest,
    ) -> Result<crate::entity::StartMatchBackfillResult, GameLiftErrorType> {
        self.http_client.backfill_matchmaking(request).await
    }

    pub async fn stop_matchmaking(
        &self,
        request: crate::entity::StopMatchBackfillRequest,
    ) -> Result<(), GameLiftErrorType> {
        self.http_client.stop_matchmaking(request).await
    }

    async fn start_health_check(&mut self) {
        log::debug!("Health check started.");

        let inner_state = self.inner.clone();
        let http_client = self.http_client.clone();
        let report_health_task = async move {
            while inner_state.is_process_ready() {
                Self::report_health(&inner_state, &http_client).await;

                tokio::time::sleep(std::time::Duration::from_secs(HEALTHCHECK_TIMEOUT_SECONDS))
                    .await;
            }
        };

        self.health_report_task = Some(tokio::spawn(report_health_task));
    }

    async fn report_health(
        inner_state: &ServerStateInner,
        http_client: &crate::http_client::HttpClient,
    ) {
        if !inner_state.is_process_ready() {
            log::debug!("Reporting Health on an inactive process. Ignoring.");
            return;
        }

        log::debug!("Reporting health using the OnHealthCheck callback.");

        let callback = (inner_state.process_parameters.lock().as_ref().unwrap().on_health_check)();
        let result = tokio::time::timeout(
            std::time::Duration::from_secs(HEALTHCHECK_TIMEOUT_SECONDS),
            callback,
        )
        .await;

        let report_health_result = if let Ok(health_check_result) = result {
            http_client.report_health(health_check_result).await
        } else {
            http_client.report_health(false).await
        };

        if let Err(error) = report_health_result {
            log::warn!("Could not send health starus: {:?}", error);
        }
    }

    pub async fn initialize_networking(
        &mut self,
        server_parameters: crate::server_parameters::ServerParameters,
    ) -> Result<(), crate::error::GameLiftErrorType> {
        let web_socket_url = std::env::var(ENVIRONMENT_VARIABLE_WEBSOCKET_URL)
            .unwrap_or(server_parameters.web_socket_url);
        self.process_id =
            std::env::var(ENVIRONMENT_VARIABLE_PROCESS_ID).unwrap_or(server_parameters.process_id);
        self.host_id =
            std::env::var(ENVIRONMENT_VARIABLE_HOST_ID).unwrap_or(server_parameters.host_id);
        self.fleet_id =
            std::env::var(ENVIRONMENT_VARIABLE_FLEET_ID).unwrap_or(server_parameters.fleet_id);
        let auth_token =
            std::env::var(ENVIRONMENT_VARIABLE_AUTH_TOKEN).unwrap_or(server_parameters.auth_token);

        self.establish_networking(web_socket_url, auth_token).await
    }

    async fn establish_networking(
        &mut self,
        web_socket_url: impl Into<String>,
        auth_token: impl Into<String>,
    ) -> Result<(), crate::error::GameLiftErrorType> {
        let server_parameters = crate::server_parameters::ServerParameters::new(
            web_socket_url,
            self.process_id.to_owned(),
            self.host_id.to_owned(),
            self.fleet_id.to_owned(),
            auth_token,
        );
        let websocket_listener = crate::web_socket_listener::WebSocketListener::connect(
            self.inner.clone(),
            server_parameters,
        )
        .await?;
        self.websocket_listener = Some(websocket_listener);
        Ok(())
    }

    pub async fn get_instance_certificate(
        &self,
    ) -> Result<GetInstanceCertificateResult, GameLiftErrorType> {
        self.http_client.get_instance_certificate().await
    }

    pub async fn shutdown(&mut self) -> bool {
        self.inner.set_is_process_ready(false);
        if let Some(health_report_task) = &self.health_report_task {
            health_report_task.abort();
        }
        std::mem::replace(&mut self.websocket_listener, None).is_some()
    }
}
