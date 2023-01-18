use futures_util::StreamExt;

const PID_KEY: &str = "pID";
const SDK_VERSION_KEY: &str = "sdkVersion";
const FLAVOR_KEY: &str = "sdkLanguage";
const FLAVOR: &str = "Rust";
const AUTH_TOKEN_KEY: &str = "Authorization";
const COMPUTE_ID_KEY: &str = "ComputeId";
const FLEET_ID_KEY: &str = "FleetId";

pub struct WebSocketListener {
    handle: tokio::task::JoinHandle<()>,
}

impl Drop for WebSocketListener {
    fn drop(&mut self) {
        self.handle.abort();
    }
}

impl WebSocketListener {
    pub async fn connect(
        state: std::sync::Arc<tokio::sync::RwLock<crate::server_state::ServerStateInner>>,
        server_parameters: crate::server_parameters::ServerParameters,
    ) -> Result<Self, crate::error::GameLiftErrorType> {
        match Self::perform_connect(state, server_parameters).await {
            Ok(handle) => Ok(Self { handle }),
            Err(error) => {
                println!("{}", error);
                Err(crate::error::GameLiftErrorType::LocalConnectionFailed)
            }
        }
    }

    async fn perform_connect(
        callback_handler: std::sync::Arc<
            tokio::sync::RwLock<crate::server_state::ServerStateInner>,
        >,
        server_parameters: crate::server_parameters::ServerParameters,
    ) -> Result<tokio::task::JoinHandle<()>, tokio_tungstenite::tungstenite::Error> {
        let connection_string = Self::create_uri(server_parameters);
        log::debug!("AWS GameLift Server WebSocket connection string: {}", connection_string);
        let (mut ws_stream, _) = tokio_tungstenite::connect_async(connection_string).await?;

        Ok(tokio::spawn(async move {
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
                                .read()
                                .await
                                .on_start_game_session(message.game_session)
                                .await;
                        }
                        ReceivedMessageType::UpdateGameSession(message) => {
                            log::info!("Received UpdateGameSession event");

                            let game_session = message.game_session.unwrap();
                            let update_reason = message.update_reason;
                            callback_handler
                                .read()
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
                                .read()
                                .await
                                .on_terminate_process(message.termination_time.unwrap())
                                .await;
                        }
                    }
                } else if msg.is_close() {
                    log::debug!("Socket disconnected. Message: {}", msg);
                }
            }
        }))
    }

    fn create_uri(server_parameters: crate::server_parameters::ServerParameters) -> String {
        let query_string = format!(
            "{}={}&{}={}&{}={}&{}={}&{}={}&{}={}",
            PID_KEY,
            server_parameters.process_id,
            SDK_VERSION_KEY,
            crate::api::Api::get_sdk_version(),
            FLAVOR_KEY,
            FLAVOR,
            AUTH_TOKEN_KEY,
            server_parameters.auth_token,
            COMPUTE_ID_KEY,
            server_parameters.host_id,
            FLEET_ID_KEY,
            server_parameters.fleet_id,
        );

        // Path to resource must end with "/"
        let web_socket_url = server_parameters.web_socket_url;
        if web_socket_url.ends_with('/') {
            format!("{}?{}", web_socket_url, query_string)
        } else {
            format!("{}/?{}", web_socket_url, query_string)
        }
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
