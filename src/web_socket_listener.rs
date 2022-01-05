use futures_util::StreamExt;

const HOSTNAME: &str = "127.0.0.1";
const PORT: i32 = 5759;
const PID_KEY: &str = "pID";
const SDK_VERSION_KEY: &str = "sdkVersion";
const FLAVOR_KEY: &str = "sdkLanguage";
const FLAVOR: &str = "Rust";

pub struct WebSocketListener {
    handle: Option<tokio::task::JoinHandle<()>>,
    state: std::sync::Arc<tokio::sync::Mutex<crate::server_state::ServerStateInner>>,
}

impl WebSocketListener {
    pub fn new(
        state: std::sync::Arc<tokio::sync::Mutex<crate::server_state::ServerStateInner>>,
    ) -> Self {
        Self { handle: None, state }
    }

    pub fn disconnect(&self) -> bool {
        if let Some(handle) = &self.handle {
            handle.abort();
            return true;
        }

        false
    }

    pub async fn connect(&mut self) -> Result<(), crate::error::GameLiftErrorType> {
        self.perform_connect().await.map_err(|error| {
            println!("{:?}", error);
            crate::error::GameLiftErrorType::LocalConnectionFailed
        })
    }

    async fn perform_connect(&mut self) -> Result<(), tokio_tungstenite::tungstenite::Error> {
        let connection_string = Self::create_uri();
        log::debug!("AWS GameLift Server WebSocket connection string: {}", connection_string);
        let (mut ws_stream, _) = tokio_tungstenite::connect_async(connection_string).await?;

        let callback_handler = self.state.clone();
        self.handle = Some(tokio::spawn(async move {
            while let Some(msg) = ws_stream.next().await {
                let msg = msg.unwrap();
                if msg.is_text() {
                    let message_text = msg.into_text().unwrap();
                    let v: serde_json::Value = serde_json::from_str(message_text.as_str()).unwrap();
                    let message_type =
                        get_inner_message_type(&v).expect("Unexpected received message");

                    match message_type {
                        ReceivedMessageType::ActivateGameSession(message) => {
                            log::info!("Received ActivateGameSession event");
                            callback_handler
                                .lock()
                                .await
                                .on_start_game_session(message.game_session)
                                .await;
                        }
                        ReceivedMessageType::UpdateGameSession(message) => {
                            log::info!("Received UpdateGameSession event");

                            let game_session = message.game_session.unwrap();
                            let update_reason = message.update_reason;
                            callback_handler
                                .lock()
                                .await
                                .on_update_game_session(
                                    game_session,
                                    update_reason,
                                    message.backfill_ticket_id,
                                )
                                .await;
                        }
                        ReceivedMessageType::TerminateProcess(message) => {
                            log::info!("Received TerminateProcess event");

                            callback_handler
                                .lock()
                                .await
                                .on_terminate_process(message.termination_time.unwrap())
                                .await;
                        }
                    }
                } else if msg.is_close() {
                    log::debug!("Socket disconnected. Message: {}", msg);
                }
            }
        }));

        Ok(())
    }

    fn create_uri() -> String {
        let query_string = format!(
            "{}={}&{}={}&{}={}",
            PID_KEY,
            std::process::id(),
            SDK_VERSION_KEY,
            crate::api::SDK_VERSION,
            FLAVOR_KEY,
            FLAVOR
        );

        format!("ws://{}:{}?{}", HOSTNAME, PORT, query_string)
    }
}

enum ReceivedMessageType {
    ActivateGameSession(crate::entity::ActivateGameSession),
    UpdateGameSession(crate::entity::UpdateGameSession),
    TerminateProcess(crate::entity::TerminateProcess),
}

fn remove_type_info_from_json(source: &serde_json::Value) -> serde_json::Value {
    match source {
        serde_json::Value::Object(m) => {
            let mut new_fields = serde_json::Map::new();
            for (k, v) in m {
                if k != "@type" {
                    new_fields.insert(k.clone(), v.clone());
                }
            }
            serde_json::Value::Object(new_fields)
        }
        v => v.clone(),
    }
}

fn get_inner_message_type(
    v: &serde_json::Value,
) -> Result<ReceivedMessageType, crate::error::GameLiftErrorType> {
    if let Some(inner_message) = v.get("innerMessage") {
        if let Some(message_type) = inner_message.get("@type") {
            if let Some(message_type) = message_type.as_str() {
                match message_type {
                    "type.googleapis.com/com.amazon.whitewater.auxproxy.pbuffer.\
                     ActivateGameSession" => {
                        let unpack_result: crate::entity::ActivateGameSession =
                            serde_json::from_value(remove_type_info_from_json(inner_message))
                                .unwrap();
                        return Ok(ReceivedMessageType::ActivateGameSession(unpack_result));
                    }
                    "type.googleapis.com/com.amazon.whitewater.auxproxy.pbuffer.\
                     UpdateGameSession" => {
                        let unpack_result: crate::entity::UpdateGameSession =
                            serde_json::from_value(remove_type_info_from_json(inner_message))
                                .unwrap();
                        return Ok(ReceivedMessageType::UpdateGameSession(unpack_result));
                    }
                    "type.googleapis.com/com.amazon.whitewater.auxproxy.pbuffer.\
                     TerminateProcess" => {
                        let unpack_result: crate::entity::TerminateProcess =
                            serde_json::from_value(remove_type_info_from_json(inner_message))
                                .unwrap();
                        return Ok(ReceivedMessageType::TerminateProcess(unpack_result));
                    }

                    _ => {}
                };
            }
        }
    }

    Err(crate::error::GameLiftErrorType::UnexpectedWebSocketMessage)
}
