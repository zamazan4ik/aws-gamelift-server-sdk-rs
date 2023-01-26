use tokio::sync::{mpsc, oneshot};
use tokio_tungstenite::tungstenite;

use crate::{
    connection_state::{ConnectionState, Feedback},
    error::Error,
    model::{
        message,
        protocol::{RequestContent, RequestMessage},
    },
    server_parameters::ServerParameters,
};

const PID_KEY: &str = "pID";
const SDK_VERSION_KEY: &str = "sdkVersion";
const FLAVOR_KEY: &str = "sdkLanguage";
const FLAVOR: &str = "Rust";
const AUTH_TOKEN_KEY: &str = "Authorization";
const COMPUTE_ID_KEY: &str = "ComputeId";
const FLEET_ID_KEY: &str = "FleetId";

pub(crate) const CHANNEL_BUFFER_SIZE: usize = 1024;
const SERVICE_CALL_TIMEOUT_MILLIS: u64 = 20000;

pub(crate) type WebSocket =
    tokio_tungstenite::WebSocketStream<tokio_tungstenite::MaybeTlsStream<tokio::net::TcpStream>>;

#[derive(Debug)]
#[allow(clippy::enum_variant_names)]
pub(crate) enum ServerEventInner {
    OnStartGameSession(message::CreateGameSessionMessage),
    OnUpdateGameSession(message::UpdateGameSessionMessage),
    OnTerminateProcess(message::TerminateProcessMessage),
    OnRefreshConnection(message::RefreshConnectionMessage),
    OnHealthCheck(),
}

#[derive(Debug)]
pub(crate) struct WebSocketListener {
    request_sender: mpsc::UnboundedSender<(RequestMessage, oneshot::Sender<Feedback>)>,
    event_receiver: Option<mpsc::Receiver<ServerEventInner>>,
}

impl WebSocketListener {
    pub(crate) async fn connect(server_parameters: &ServerParameters) -> Result<Self, Error> {
        let connection_string = Self::create_uri(server_parameters);
        log::debug!("AWS GameLift Server WebSocket connection uri: {}", connection_string);
        match tokio_tungstenite::connect_async(connection_string).await {
            Ok((web_socket, _)) => {
                let (request_sender, request_receiver) = mpsc::unbounded_channel();
                let (event_sender, event_receiver) = mpsc::channel(CHANNEL_BUFFER_SIZE);
                let connection_state =
                    ConnectionState::new(web_socket, request_receiver, event_sender);
                tokio::spawn(connection_state.run());
                log::info!("Connected to GameLift API Gateway.");
                Ok(Self { request_sender, event_receiver: Some(event_receiver) })
            }
            Err(error) => Err(Error::LocalConnectionFailed(error)),
        }
    }

    pub(crate) async fn request<T>(
        &self,
        message: T,
    ) -> Result<<T as RequestContent>::Response, Error>
    where
        T: RequestContent,
    {
        let message = RequestMessage {
            action: T::ACTION_NAME.to_owned(),
            request_id: uuid::Uuid::new_v4().to_string(),
            content: serde_json::to_value(message)?,
        };
        let (feedback_sender, feedback_receiver) = oneshot::channel();
        self.request_sender
            .send((message, feedback_sender))
            .map_err(|_| Error::LocalConnectionAlreadyClosed)?;
        let result = tokio::time::timeout(
            std::time::Duration::from_millis(SERVICE_CALL_TIMEOUT_MILLIS),
            feedback_receiver,
        )
        .await
        .map_err(|_| Error::RequestTimeout)?
        .map_err(|_| Error::LocalConnectionAlreadyClosed)??;

        if result.status_code != tungstenite::http::StatusCode::OK.as_u16() {
            Err(Error::RequestUnsuccessful(result.status_code, result.error_message))
        } else {
            let mut rest_data = result.rest_data;
            if let serde_json::Value::Object(obj) = &rest_data {
                if obj.is_empty() {
                    // Allow conversion to ()
                    rest_data = serde_json::Value::Null;
                }
            }
            Ok(serde_json::from_value(rest_data)?)
        }
    }

    pub(crate) fn take_event_receiver(&mut self) -> Option<mpsc::Receiver<ServerEventInner>> {
        self.event_receiver.take()
    }

    fn create_uri(server_parameters: &ServerParameters) -> String {
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
        let web_socket_url = &server_parameters.web_socket_url;
        if web_socket_url.ends_with('/') {
            format!("{}?{}", web_socket_url, query_string)
        } else {
            format!("{}/?{}", web_socket_url, query_string)
        }
    }
}
