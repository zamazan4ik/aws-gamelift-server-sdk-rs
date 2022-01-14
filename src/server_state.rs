use crate::{entity::GetInstanceCertificateResult, error::GameLiftErrorType};
use tokio::task::JoinHandle;

const HEALTHCHECK_TIMEOUT_SECONDS: u64 = 60;

#[derive(Default)]
struct SessionState {
    is_process_ready: bool,
    game_session_id: Option<crate::entity::GameSessionId>,
    termination_time: Option<crate::entity::TerminationTimeType>,
}

#[derive(Default)]
pub struct ServerStateInner {
    process_parameters: Option<crate::process_parameters::ProcessParameters>,
    session_state: parking_lot::RwLock<SessionState>,
    http_client: crate::http_client::HttpClient,
}

impl ServerStateInner {
    pub fn is_process_ready(&self) -> bool {
        self.session_state.read().is_process_ready
    }

    pub fn get_game_session_id(&self) -> Option<crate::entity::GameSessionId> {
        self.session_state.read().game_session_id.clone()
    }

    pub fn get_termination_time(&self) -> Option<crate::entity::TerminationTimeType> {
        self.session_state.read().termination_time
    }

    pub async fn on_start_game_session(&self, game_session: crate::entity::GameSession) {
        if !self.is_process_ready() {
            log::debug!("Got a game session on inactive process. Ignoring.");
            return;
        }

        self.session_state.write().game_session_id =
            Some(game_session.game_session_id.clone().unwrap());
        (self.process_parameters.as_ref().unwrap().on_start_game_session)(game_session).await;
    }

    pub async fn on_terminate_process(&self, termination_time: i64) {
        log::debug!(
            "ServerState got the terminateProcess signal. TerminateProcess: {}",
            termination_time
        );
        self.session_state.write().termination_time = Some(termination_time);
        (self.process_parameters.as_ref().unwrap().on_process_terminate)().await;
    }

    pub async fn on_update_game_session(
        &self,
        game_session: crate::entity::GameSession,
        update_reason: crate::entity::UpdateReason,
        backfill_ticket_id: String,
    ) {
        if !self.is_process_ready() {
            log::warn!("Got an updated game session on inactive process.");
            return;
        }
        (self.process_parameters.as_ref().unwrap().on_update_game_session)(
            crate::entity::UpdateGameSession {
                game_session: Some(game_session),
                update_reason,
                backfill_ticket_id,
            },
        )
        .await;
    }

    pub async fn report_health(&self) {
        if !self.is_process_ready() {
            log::debug!("Reporting Health on an inactive process. Ignoring.");
            return;
        }

        log::debug!("Reporting health using the OnHealthCheck callback.");

        let result = tokio::time::timeout(
            std::time::Duration::from_secs(HEALTHCHECK_TIMEOUT_SECONDS),
            (self.process_parameters.as_ref().unwrap().on_health_check)(),
        )
        .await;

        let report_health_result = if let Ok(health_check_result) = result {
            self.http_client.report_health(health_check_result).await
        } else {
            self.http_client.report_health(false).await
        };

        if let Err(error) = report_health_result {
            log::warn!("Could not send health starus: {:?}", error);
        }
    }
}

pub struct ServerState {
    inner: std::sync::Arc<tokio::sync::RwLock<ServerStateInner>>,
    websocket_listener: Option<crate::web_socket_listener::WebSocketListener>,
    health_report_task: Option<JoinHandle<()>>,
}

impl Default for ServerState {
    fn default() -> Self {
        Self {
            inner: std::sync::Arc::new(tokio::sync::RwLock::new(ServerStateInner::default())),
            websocket_listener: None,
            health_report_task: None,
        }
    }
}

impl ServerState {
    pub async fn process_ready(
        &mut self,
        process_parameters: crate::process_parameters::ProcessParameters,
    ) -> Result<(), crate::error::GameLiftErrorType> {
        let port = process_parameters.port;
        let log_paths = process_parameters.log_parameters.log_paths.clone();

        let result = {
            let mut inner = self.inner.write().await;

            inner.session_state.write().is_process_ready = true;
            inner.process_parameters = Some(process_parameters);

            inner.http_client.process_ready(port, log_paths).await
        };

        self.start_health_check().await;

        result
    }

    pub async fn process_ending(&self) -> Result<(), crate::error::GameLiftErrorType> {
        let inner = self.inner.read().await;

        inner.session_state.write().is_process_ready = false;
        inner.http_client.process_ending().await
    }

    pub async fn activate_game_session(&self) -> Result<(), GameLiftErrorType> {
        let inner = self.inner.read().await;

        let game_session_id = inner.get_game_session_id();
        if let Some(game_session_id) = game_session_id {
            inner.http_client.activate_game_session(game_session_id).await
        } else {
            Err(crate::error::GameLiftErrorType::GameSessionIdNotSet)
        }
    }

    pub async fn terminate_game_session(&self) -> Result<(), GameLiftErrorType> {
        let inner = self.inner.read().await;

        let game_session_id = inner.get_game_session_id();
        if let Some(game_session_id) = game_session_id {
            inner.http_client.terminate_game_session(game_session_id).await
        } else {
            Err(crate::error::GameLiftErrorType::GameSessionIdNotSet)
        }
    }

    pub async fn get_game_session_id(
        &self,
    ) -> Result<crate::entity::GameSessionId, crate::error::GameLiftErrorType> {
        match self.inner.read().await.get_game_session_id() {
            Some(game_session_id) => Ok(game_session_id),
            None => Err(crate::error::GameLiftErrorType::GameSessionIdNotSet),
        }
    }

    pub async fn get_termination_time(
        &self,
    ) -> Result<crate::entity::TerminationTimeType, crate::error::GameLiftErrorType> {
        match self.inner.read().await.get_termination_time() {
            Some(value) => Ok(value),
            None => Err(crate::error::GameLiftErrorType::TerminationTimeNotSet),
        }
    }

    pub async fn update_player_session_creation_policy(
        &self,
        player_session_policy: crate::entity::PlayerSessionCreationPolicy,
    ) -> Result<(), GameLiftErrorType> {
        let inner = self.inner.read().await;

        let game_session_id = inner.get_game_session_id();
        if let Some(game_session_id) = game_session_id {
            inner
                .http_client
                .update_player_session_creation_policy(game_session_id, player_session_policy)
                .await
        } else {
            Err(crate::error::GameLiftErrorType::GameSessionIdNotSet)
        }
    }

    pub async fn accept_player_session(
        &self,
        player_session_id: crate::entity::PlayerSessionId,
    ) -> Result<(), GameLiftErrorType> {
        let inner = self.inner.read().await;

        let game_session_id = inner.get_game_session_id();
        if let Some(game_session_id) = game_session_id {
            inner.http_client.accept_player_session(player_session_id, game_session_id).await
        } else {
            Err(crate::error::GameLiftErrorType::GameSessionIdNotSet)
        }
    }

    pub async fn remove_player_session(
        &self,
        player_session_id: crate::entity::PlayerSessionId,
    ) -> Result<(), GameLiftErrorType> {
        let inner = self.inner.read().await;

        let game_session_id = inner.get_game_session_id();
        if let Some(game_session_id) = game_session_id {
            inner.http_client.remove_player_session(player_session_id, game_session_id).await
        } else {
            Err(crate::error::GameLiftErrorType::GameSessionIdNotSet)
        }
    }

    pub async fn describe_player_sessions(
        &self,
        request: crate::entity::DescribePlayerSessionsRequest,
    ) -> Result<crate::entity::DescribePlayerSessionsResult, GameLiftErrorType> {
        self.inner.read().await.http_client.describe_player_sessions(request).await
    }

    pub async fn backfill_matchmaking(
        &self,
        request: crate::entity::StartMatchBackfillRequest,
    ) -> Result<crate::entity::StartMatchBackfillResult, GameLiftErrorType> {
        self.inner.read().await.http_client.backfill_matchmaking(request).await
    }

    pub async fn stop_matchmaking(
        &self,
        request: crate::entity::StopMatchBackfillRequest,
    ) -> Result<(), GameLiftErrorType> {
        self.inner.read().await.http_client.stop_matchmaking(request).await
    }

    async fn start_health_check(&mut self) {
        log::debug!("Health check started.");

        let inner_state = self.inner.clone();
        let report_health_task = async move {
            while inner_state.read().await.is_process_ready() {
                {
                    inner_state.read().await.report_health().await;
                }

                tokio::time::sleep(std::time::Duration::from_secs(HEALTHCHECK_TIMEOUT_SECONDS))
                    .await;
            }
        };

        self.health_report_task = Some(tokio::spawn(report_health_task));
    }

    pub async fn initialize_networking(&mut self) -> Result<(), crate::error::GameLiftErrorType> {
        self.websocket_listener =
            Some(crate::web_socket_listener::WebSocketListener::new(self.inner.clone()));
        self.websocket_listener.as_mut().unwrap().connect().await
    }

    pub async fn get_instance_certificate(
        &self,
    ) -> Result<GetInstanceCertificateResult, GameLiftErrorType> {
        self.inner.read().await.http_client.get_instance_certificate().await
    }

    pub async fn shutdown(&self) -> bool {
        self.inner.read().await.session_state.write().is_process_ready = false;
        if let Some(health_report_task) = &self.health_report_task {
            health_report_task.abort();
        }
        self.websocket_listener.as_ref().unwrap().disconnect()
    }
}
