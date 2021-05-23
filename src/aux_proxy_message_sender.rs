const MESSAGE_TYPE_PREFIX: &'static str = "com.amazon.whitewater.auxproxy.pbuffer";

pub struct AuxProxyMessageSender {
    socket: rust_socketio::Socket,
}

impl AuxProxyMessageSender {
    pub fn new(socket: rust_socketio::Socket) -> Self {
        Self { socket }
    }

    pub fn process_ready(&mut self, port: i32, log_paths_to_upload: Vec<String>) {
        let mut message = crate::sdk::ProcessReady::default();
        message.port = port;
        message.log_paths_to_upload = log_paths_to_upload;

        self.send(message);
    }

    pub fn process_ending(&mut self) {
        self.send(crate::sdk::ProcessEnding::default());
    }

    pub fn activate_game_session(&mut self, game_session_id: crate::entity::GameSessionId) {
        let mut message = crate::sdk::GameSessionActivate::default();
        message.game_session_id = game_session_id;

        self.send(message);
    }

    pub fn terminate_game_session(&mut self, game_session_id: crate::entity::GameSessionId) {
        let mut message = crate::sdk::GameSessionTerminate::default();
        message.game_session_id = game_session_id;

        self.send(message);
    }

    pub fn update_player_session_creation_policy(
        &mut self,
        game_session_id: crate::entity::GameSessionId,
        player_session_policy: crate::entity::PlayerSessionCreationPolicy,
    ) {
        let mut message = crate::sdk::UpdatePlayerSessionCreationPolicy::default();
        message.game_session_id = game_session_id;
        message.new_player_session_creation_policy = player_session_policy.to_string();

        self.send(message);
    }

    pub fn accept_player_session(
        &mut self,
        player_session_id: crate::entity::PlayerSessionId,
        game_session_id: crate::entity::GameSessionId,
    ) {
        let mut message = crate::sdk::AcceptPlayerSession::default();
        message.player_session_id = player_session_id;
        message.game_session_id = game_session_id;

        self.send(message);
    }

    pub fn remove_player_session(
        &mut self,
        player_session_id: crate::entity::PlayerSessionId,
        game_session_id: crate::entity::GameSessionId,
    ) {
        let mut message = crate::sdk::RemovePlayerSession::default();
        message.player_session_id = player_session_id;
        message.game_session_id = game_session_id;

        self.send(message);
    }

    pub fn backfill_matchmaking(&mut self, request: crate::entity::StartMatchBackfillRequest) {
        self.send(crate::mapper::start_match_backfill_request_mapper(request));
    }

    /*public BackfillMatchmaking(
    request: StartMatchBackfillRequest
    ): Promise<StartMatchBackfillOutcome> {
    const translation = BackfillDataMapper.CreateBufferedBackfillMatchmakingRequest(request)

    const deferred = pDefer<StartMatchBackfillOutcome>()

    const ackFunction = this.CreateAckFunctionForStartMatchBackfill(deferred)

    return this.EmitEvent(
    translation,
    ackFunction,
    deferred,
    AuxProxyMessageSender.START_MATCH_BACKFILL_ERROR
    )
    }*/

    pub fn stop_matchmaking(&mut self, request: crate::entity::StopMatchBackfillRequest) {
        self.send(crate::mapper::stop_matchmaking_request_mapper(request));
    }

    pub fn report_health(&mut self, health_status: bool) {
        let mut message = crate::sdk::ReportHealth::default();
        message.health_status = health_status;

        self.send(message);
    }

    fn send<T>(&mut self, message: T)
    where
        T: serde::Serialize,
    {
        let json_payload = serde_json::to_string(&message).unwrap();
        self.socket
            .emit(
                format!("{}.{}", MESSAGE_TYPE_PREFIX, get_message_type(&message)),
                json_payload,
            )
            .expect("Server unreachable");
    }
}

fn get_message_type<T>(_: &T) -> &str {
    let full_name = std::any::type_name::<T>();
    &full_name[full_name.rfind(':').unwrap() + 1..]
}

#[cfg(test)]
mod tests {
    use crate::aux_proxy_message_sender::{get_message_type, MESSAGE_TYPE_PREFIX};

    pub mod sdk {
        include!(concat!(
            env!("OUT_DIR"),
            "/com.amazon.whitewater.auxproxy.pbuffer.rs"
        ));
    }

    #[test]
    fn it_works() {
        let process_ready = sdk::ProcessReady::default();

        assert_eq!(
            format!(
                "{}.{}",
                MESSAGE_TYPE_PREFIX,
                get_message_type(&process_ready)
            ),
            "com.amazon.whitewater.auxproxy.pbuffer.ProcessReady"
        );
    }
}
