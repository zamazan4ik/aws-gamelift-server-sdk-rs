use std::collections::HashMap;

use futures_util::{SinkExt, StreamExt};
use tokio::sync::{mpsc, oneshot};
use tokio_tungstenite::tungstenite;

use crate::{
    model::{
        self,
        protocol::{self, RequestContent, RequestMessage, ResponceMessage},
    },
    web_socket_listener::{ServerEventInner, WebSocket},
    Error,
};

pub(crate) type Feedback = Result<ResponceMessage, Error>;

pub(crate) struct ConnectionState {
    web_socket: WebSocket,
    request_receiver: mpsc::UnboundedReceiver<(RequestMessage, oneshot::Sender<Feedback>)>,
    event_sender: mpsc::Sender<ServerEventInner>,
    requests: HashMap<String, oneshot::Sender<Feedback>>,
    terminate_request_id: Option<String>,
}

impl ConnectionState {
    pub(crate) fn new(
        web_socket: WebSocket,
        request_receiver: mpsc::UnboundedReceiver<(RequestMessage, oneshot::Sender<Feedback>)>,
        event_sender: mpsc::Sender<ServerEventInner>,
    ) -> Self {
        Self {
            web_socket,
            request_receiver,
            event_sender,
            requests: HashMap::new(),
            terminate_request_id: None,
        }
    }

    pub(crate) async fn run(mut self) {
        let mut request_closed = false;
        loop {
            let transition_close = tokio::select! {
                request = self.request_receiver.recv(), if !request_closed => self.send_request(request).await,
                Some(next) = self.web_socket.next() => self.listen(next).await,
                else => break,
            };
            if transition_close {
                request_closed = true;
            }
        }

        // TerminateServerProcess action responds with close request instead of success
        if let Some(id) = self.terminate_request_id.take() {
            self.do_feedback(protocol::ResponceMessage::new_fake_success(
                model::request::TerminateServerProcessRequest::ACTION_NAME.to_owned(),
                id,
            ));
        }

        log::info!("Websocket closed");
    }

    async fn send_request(
        &mut self,
        request: Option<(RequestMessage, oneshot::Sender<Feedback>)>,
    ) -> bool {
        if let Some((msg, feedback)) = request {
            log::info!("Sending {}: request_id {}", msg.action, msg.request_id);
            match serde_json::to_string(&msg) {
                Ok(json) => {
                    log::debug!("Sending socket message: \n{json}");

                    if let Err(error) = self.web_socket.send(tungstenite::Message::Text(json)).await
                    {
                        Self::do_feedback_error(feedback, msg.request_id, error);
                        return false;
                    };
                    if msg.action == model::request::TerminateServerProcessRequest::ACTION_NAME {
                        self.terminate_request_id = Some(msg.request_id.clone());
                    }
                    if let Some(prev) = self.requests.insert(msg.request_id.clone(), feedback) {
                        Self::do_feedback_error(prev, msg.request_id, Error::RequestWasOverwritten);
                    }
                }
                Err(error) => {
                    Self::do_feedback_error(feedback, msg.request_id, error);
                }
            };

            false
        } else {
            if let Err(e) = self.web_socket.close(None).await {
                log::warn!("Close request error: {e}");
            };
            true
        }
    }

    async fn listen(&mut self, next: Result<tungstenite::Message, tungstenite::Error>) -> bool {
        match next {
            Ok(msg) => match msg {
                tungstenite::Message::Text(t) => {
                    if let Err(e) = self.reaction(t).await {
                        log::warn!("Reaction error: {e}");
                    }
                }
                tungstenite::Message::Binary(b) => {
                    log::warn!("Received unknown binary massage: {b:?}")
                }
                tungstenite::Message::Ping(p) => log::trace!("Received ping: {p:?}"),
                tungstenite::Message::Pong(p) => log::trace!("Received pong: {p:?}"),
                tungstenite::Message::Close(cf) => {
                    if let Some(cf) = cf {
                        log::info!("Received close request: {cf}");
                    } else {
                        log::info!("Received close request");
                    }
                    self.request_receiver.close();
                    return true;
                }
                tungstenite::Message::Frame(f) => log::warn!("Received unknown raw frame: {f}"),
            },
            Err(e) => {
                log::error!("Message receive error: {e}");
            }
        }
        false
    }

    async fn reaction(&mut self, json: String) -> Result<(), Box<dyn std::error::Error>> {
        log::debug!("Received socket message: \n{}", &json);
        let message: ResponceMessage = serde_json::from_str(&json)?;
        if message.status_code != tungstenite::http::StatusCode::OK.as_u16()
            && !message.request_id.is_empty()
        {
            log::info!(
                "Received {} is unsuccessful with status code {}: request_id {}, error_message '{}'",
                message.action,
                message.status_code,
                message.request_id,
                message.error_message
            );
        } else {
            log::info!(
                "Received {} with status code {}: request_id {}",
                message.action,
                message.status_code,
                message.request_id
            );
        }

        // Use try_send() to secure against DoS attacks (I don't think it makes much sense)
        match message.action.as_str() {
            model::message::CreateGameSessionMessage::ACTION_NAME => {
                let data: model::message::CreateGameSessionMessage =
                    serde_json::from_value(message.rest_data)?;
                self.event_sender.try_send(ServerEventInner::OnStartGameSession(data))?;
            }
            model::message::RefreshConnectionMessage::ACTION_NAME => {
                self.request_receiver.close();
                let data: model::message::RefreshConnectionMessage =
                    serde_json::from_value(message.rest_data)?;
                self.event_sender.try_send(ServerEventInner::OnRefreshConnection(data))?;
            }
            model::message::TerminateProcessMessage::ACTION_NAME => {
                let data: model::message::TerminateProcessMessage =
                    serde_json::from_value(message.rest_data)?;
                self.event_sender.try_send(ServerEventInner::OnTerminateProcess(data))?;
            }
            model::message::UpdateGameSessionMessage::ACTION_NAME => {
                let data: model::message::UpdateGameSessionMessage =
                    serde_json::from_value(message.rest_data)?;
                self.event_sender.try_send(ServerEventInner::OnUpdateGameSession(data))?;
            }
            _ => {
                self.do_feedback(message);
            }
        }
        Ok(())
    }

    fn do_feedback(&mut self, message: ResponceMessage) {
        if message.request_id.is_empty() {
            log::warn!("Request id was null with {}", message.action);
            return;
        }

        match self.requests.remove(&message.request_id) {
            Some(feedback) => {
                if let Err(Ok(v)) = feedback.send(Ok(message)) {
                    log::warn!("Responces could not be feedbacked: request_id {}", v.request_id);
                }
            }
            None => {
                log::warn!("Request {} not found", message.request_id);
            }
        }
    }

    fn do_feedback_error(
        feedback: oneshot::Sender<Feedback>,
        request_id: String,
        error: impl Into<Error>,
    ) {
        if let Err(Err(v)) = feedback.send(Err(error.into())) {
            log::warn!("Errors could not be feedbacked: request_id {request_id}, error {v}");
        }
    }
}
