use protobuf::RepeatedField;
use reqwest::Response;

const MESSAGE_TYPE_PREFIX: &'static str = "com.amazon.whitewater.auxproxy.pbuffer";

pub struct HttpClient {
    uri: reqwest::Url,
    http_client: reqwest::Client,
}

impl HttpClient {
    pub fn new() -> Self {
        use reqwest::header;
        let mut headers = header::HeaderMap::new();
        headers.insert(
            header::ACCEPT,
            header::HeaderValue::from_static("application/json"),
        );
        headers.insert(
            header::HeaderName::from_static("gamelift-server-pid"),
            header::HeaderValue::from_str(std::process::id().to_string().as_str()).unwrap(),
        );

        Self {
            uri: reqwest::Url::parse("http://localhost:5758/")
                .expect("Cannot parse GameLift Server URI"),
            http_client: reqwest::ClientBuilder::new()
                .default_headers(headers)
                .build()
                .expect("Cannot build HTTP client"),
        }
    }

    async fn send<T>(&self, message: T) -> Result<Response, crate::error::GameLiftErrorType>
    where
        T: protobuf::Message,
    {
        /*let mut buf = bytes::BytesMut::new();
        buf.resize(message.encoded_len(), 0);
        message.encode(&mut buf);*/

        self.http_client
            .post(self.uri.clone())
            .header(
                "gamelift-target",
                format!("{}.{}", MESSAGE_TYPE_PREFIX, get_message_type(&message)),
            )
            .body(message.write_to_bytes().unwrap())
            .send()
            .await
            .map_err(|error| {
                if error.status().is_some() && error.status().unwrap().is_server_error() {
                    crate::error::GameLiftErrorType::InternalServiceError
                } else {
                    crate::error::GameLiftErrorType::BadRequest
                }
            })
    }

    pub async fn process_ready(
        &self,
        port: i32,
        log_paths_to_upload: Vec<String>,
    ) -> Result<(), crate::error::GameLiftErrorType> {
        let mut message = crate::protos::generated_with_pure::sdk::ProcessReady::default();
        message.port = port;
        message.logPathsToUpload = RepeatedField::from_vec(log_paths_to_upload);

        self.send(message).await.map(|_| ())
    }

    pub async fn process_ending(&self) -> Result<(), crate::error::GameLiftErrorType> {
        self.send(crate::protos::generated_with_pure::sdk::ProcessEnding::default())
            .await
            .map(|_| ())
    }

    pub async fn report_health(
        &self,
        health_status: bool,
    ) -> Result<(), crate::error::GameLiftErrorType> {
        let mut message = crate::protos::generated_with_pure::sdk::ReportHealth::default();
        message.healthStatus = health_status;

        self.send(message).await.map(|_| ())
    }

    pub async fn activate_game_session(
        &self,
        game_session_id: crate::entity::GameSessionId,
    ) -> Result<(), crate::error::GameLiftErrorType> {
        let mut message = crate::protos::generated_with_pure::sdk::GameSessionActivate::default();
        message.gameSessionId = game_session_id;

        self.send(message).await.map(|_| ())
    }

    pub async fn terminate_game_session(
        &self,
        game_session_id: crate::entity::GameSessionId,
    ) -> Result<(), crate::error::GameLiftErrorType> {
        let mut message = crate::protos::generated_with_pure::sdk::GameSessionTerminate::default();
        message.gameSessionId = game_session_id;

        self.send(message).await.map(|_| ())
    }

    pub async fn update_player_session_creation_policy(
        &self,
        game_session_id: crate::entity::GameSessionId,
        player_session_policy: crate::entity::PlayerSessionCreationPolicy,
    ) -> Result<(), crate::error::GameLiftErrorType> {
        let mut message =
            crate::protos::generated_with_pure::sdk::UpdatePlayerSessionCreationPolicy::default();
        message.gameSessionId = game_session_id;
        message.newPlayerSessionCreationPolicy = player_session_policy.to_string();

        self.send(message).await.map(|_| ())
    }

    pub async fn accept_player_session(
        &self,
        player_session_id: crate::entity::PlayerSessionId,
        game_session_id: crate::entity::GameSessionId,
    ) -> Result<(), crate::error::GameLiftErrorType> {
        let mut message = crate::protos::generated_with_pure::sdk::AcceptPlayerSession::default();
        message.playerSessionId = player_session_id;
        message.gameSessionId = game_session_id;

        self.send(message).await.map(|_| ())
    }

    pub async fn remove_player_session(
        &self,
        player_session_id: crate::entity::PlayerSessionId,
        game_session_id: crate::entity::GameSessionId,
    ) -> Result<(), crate::error::GameLiftErrorType> {
        let mut message = crate::protos::generated_with_pure::sdk::RemovePlayerSession::default();
        message.playerSessionId = player_session_id;
        message.gameSessionId = game_session_id;

        self.send(message).await.map(|_| ())
    }

    pub async fn describe_player_sessions(
        &self,
        request: crate::entity::DescribePlayerSessionsRequest,
    ) -> Result<crate::entity::DescribePlayerSessionsResult, crate::error::GameLiftErrorType> {
        let response = self
            .send(crate::mapper::describe_player_sessions_mapper(request))
            .await;

        match response {
            Ok(response) => {
                let proto_response: crate::protos::generated_with_pure::sdk::DescribePlayerSessionsResponse =
                    serde_json::from_str(response.text().await.unwrap().as_str()).unwrap();
                Ok(crate::mapper::describe_player_session_request_mapper(
                    proto_response,
                ))
            }
            Err(error) => Err(error),
        }
    }

    pub async fn backfill_matchmaking(
        &self,
        request: crate::entity::StartMatchBackfillRequest,
    ) -> Result<crate::entity::StartMatchBackfillResult, crate::error::GameLiftErrorType> {
        let response = self
            .send(crate::mapper::start_match_backfill_request_mapper(request))
            .await;

        match response {
            Ok(response) => {
                let p: crate::protos::generated_with_pure::sdk::BackfillMatchmakingResponse =
                    serde_json::from_str(response.text().await.unwrap().as_str()).unwrap();
                Ok(crate::mapper::start_matchmaking_result_mapper(p))
            }
            Err(error) => Err(error),
        }
    }

    pub async fn stop_matchmaking(
        &self,
        request: crate::entity::StopMatchBackfillRequest,
    ) -> Result<(), crate::error::GameLiftErrorType> {
        self.send(crate::mapper::stop_matchmaking_request_mapper(request))
            .await
            .map(|_| ())
    }

    pub async fn get_instance_certificate(
        &self,
    ) -> Result<crate::entity::GetInstanceCertificateResult, crate::error::GameLiftErrorType> {
        let response = self
            .send(crate::protos::generated_with_pure::sdk::GetInstanceCertificate::default())
            .await;

        match response {
            Ok(response) => {
                let p: crate::protos::generated_with_pure::sdk::GetInstanceCertificateResponse =
                    serde_json::from_str(response.text().await.unwrap().as_str()).unwrap();
                Ok(crate::mapper::get_instance_certificate_result_mapper(p))
            }
            Err(error) => Err(error),
        }
    }
}

fn get_message_type<T>(_: &T) -> &str {
    let full_name = std::any::type_name::<T>();
    &full_name[full_name.rfind(':').unwrap() + 1..]
}

/*#[cfg(test)]
mod tests {
    use crate::http_client::{get_message_type, MESSAGE_TYPE_PREFIX};

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
}*/
