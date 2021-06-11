use futures_util::StreamExt;

const HOSTNAME: &'static str = "127.0.0.1";
const PORT: i32 = 5759;
const PID_KEY: &'static str = "pID";
const SDK_VERSION_KEY: &'static str = "sdkVersion";
const FLAVOR_KEY: &'static str = "sdkLanguage";
const FLAVOR: &'static str = "Rust";

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

        return false;
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
                    let p: crate::protos::generated_with_pure::sdk::AuxProxyToSdkEnvelope =
                        serde_json::from_str(message_text.as_str()).unwrap();
                    let inner_message = p.inner_message.unwrap();
                    if inner_message
                        .is::<crate::protos::generated_with_pure::sdk::ActivateGameSession>()
                    {
                        log::info!("Received ActivateGameSession event");
                        let unpack_result =
                            inner_message
                                .unpack::<crate::protos::generated_with_pure::sdk::ActivateGameSession>(
                                );
                        match unpack_result {
                            Ok(value) => {
                                if let Some(value) = value {
                                    let game_session = crate::mapper::game_session_mapper(
                                        value.gameSession.unwrap(),
                                    );
                                    callback_handler
                                        .lock()
                                        .await
                                        .on_start_game_session(game_session);
                                } else {
                                    log::error!(
                                        "Type mismatch: cannot parse as ActivateGameSession event"
                                    )
                                }
                            }
                            Err(error) => {
                                log::error!("Cannot parse ActivateGameSession event: {}", error)
                            }
                        }
                    } else if inner_message
                        .is::<crate::protos::generated_with_pure::sdk::UpdateGameSession>()
                    {
                        log::info!("Received UpdateGameSession event");
                        let unpack_result = inner_message
                            .unpack::<crate::protos::generated_with_pure::sdk::UpdateGameSession>(
                            );

                        match unpack_result {
                            Ok(value) => {
                                if let Some(value) = value {
                                    use std::str::FromStr;

                                    let game_session = crate::mapper::game_session_mapper(
                                        value.gameSession.unwrap(),
                                    );
                                    let update_reason =
                                        crate::entity::UpdateReason::from_str(&value.updateReason)
                                            .unwrap();
                                    callback_handler.lock().await.on_update_game_session(
                                        game_session,
                                        update_reason,
                                        value.backfillTicketId,
                                    );
                                } else {
                                    log::error!(
                                        "Type mismatch: cannot parse as UpdateGameSession event"
                                    )
                                }
                            }
                            Err(error) => {
                                log::error!("Cannot parse UpdateGameSession event: {}", error)
                            }
                        }
                    } else if inner_message
                        .is::<crate::protos::generated_with_pure::sdk::TerminateProcess>()
                    {
                        log::info!("Received TerminateProcess event");
                        let unpack_result = inner_message
                            .unpack::<crate::protos::generated_with_pure::sdk::TerminateProcess>(
                            );

                        match unpack_result {
                            Ok(value) => {
                                if let Some(value) = value {
                                    callback_handler
                                        .lock()
                                        .await
                                        .on_terminate_process(value.terminationTime);
                                } else {
                                    log::error!(
                                        "Type mismatch: cannot parse as TerminateProcess event"
                                    )
                                }
                            }
                            Err(error) => {
                                log::error!("Cannot parse TerminateProcess event: {}", error)
                            }
                        }
                    } else {
                        log::error!("Unknown message type received. Data is \n{}", message_text);
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
