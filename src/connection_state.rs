use std::collections::HashMap;

use futures_util::{SinkExt, StreamExt};
use tokio::sync::{mpsc, oneshot};
use tokio_tungstenite::tungstenite;

use crate::{
    model::{
        self,
        protocol::{self, RequestContent, RequestMessage, ResponceMessage},
    },
    web_socket_listener::{GameLiftEventInner, WebSocket},
};

pub(crate) struct ConnectionState {
    web_socket: WebSocket,
    request_receiver: mpsc::UnboundedReceiver<(RequestMessage, oneshot::Sender<ResponceMessage>)>,
    event_sender: mpsc::Sender<GameLiftEventInner>,
    requests: HashMap<String, oneshot::Sender<ResponceMessage>>,
    terminate_request_id: Option<String>,
}

impl ConnectionState {
    pub(crate) fn new(
        web_socket: WebSocket,
        request_receiver: mpsc::UnboundedReceiver<(
            RequestMessage,
            oneshot::Sender<ResponceMessage>,
        )>,
        event_sender: mpsc::Sender<GameLiftEventInner>,
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
        request: Option<(RequestMessage, oneshot::Sender<ResponceMessage>)>,
    ) -> bool {
        if let Some((msg, feedback)) = request {
            log::info!("Sending {}: request_id {}", msg.action, msg.request_id);
            let json = match serde_json::to_string(&msg) {
                Ok(v) => v,
                Err(e) => {
                    log::warn!("Invalid request error: {e}");
                    return false;
                }
            };
            log::debug!("Sending socket message: \n{json}");

            match self.web_socket.send(tungstenite::Message::Text(json)).await {
                Ok(_) => {
                    if msg.action == model::request::TerminateServerProcessRequest::ACTION_NAME {
                        self.terminate_request_id = Some(msg.request_id.clone());
                    }
                    if let Some(_prev) = self.requests.insert(msg.request_id, feedback) {
                        log::warn!("One request was overwritten");
                    }
                }
                Err(e) => {
                    log::warn!("Request send error: {e}");
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
                        log::warn!("{e}");
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
                self.event_sender.try_send(GameLiftEventInner::OnStartGameSession(data))?;
            }
            model::message::RefreshConnectionMessage::ACTION_NAME => {
                let data: model::message::RefreshConnectionMessage =
                    serde_json::from_value(message.rest_data)?;
                self.event_sender.try_send(GameLiftEventInner::OnRefreshConnection(data))?;
            }
            model::message::TerminateProcessMessage::ACTION_NAME => {
                let data: model::message::TerminateProcessMessage =
                    serde_json::from_value(message.rest_data)?;
                self.event_sender.try_send(GameLiftEventInner::OnTerminateProcess(data))?;
            }
            model::message::UpdateGameSessionMessage::ACTION_NAME => {
                let data: model::message::UpdateGameSessionMessage =
                    serde_json::from_value(message.rest_data)?;
                self.event_sender.try_send(GameLiftEventInner::OnUpdateGameSession(data))?;
            }
            _ => {
                if message.request_id.is_empty() {
                    log::info!("Request id was null with {}", message.action);
                } else {
                    self.do_feedback(message);
                }
            }
        }
        Ok(())
    }

    fn do_feedback(&mut self, message: ResponceMessage) {
        match self.requests.remove(&message.request_id) {
            Some(feedback) => {
                if let Err(e) = feedback.send(message) {
                    log::warn!("Feedback error {}", e.request_id);
                }
            }
            None => {
                log::warn!("Request {} not found", message.request_id);
            }
        }
    }
}
