use futures_util::SinkExt;
use tokio_tungstenite::{MaybeTlsStream, WebSocketStream};

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

/*public ReportHealth(healthStatus: boolean): Promise<GenericOutcome> {
const rHealth = new sdk.com.amazon.whitewater.auxproxy.pbuffer.ReportHealth()
rHealth.healthStatus = healthStatus

return this.EmitEventGeneric(rHealth)
}
*/

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
        let mut process_ready = sdk::ProcessReady::default();

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
