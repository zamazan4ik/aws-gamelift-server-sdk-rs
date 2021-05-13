use futures_util::SinkExt;
use tokio_tungstenite::{MaybeTlsStream, WebSocketStream};

pub mod sdk {
    include!(concat!(
        env!("OUT_DIR"),
        "/com.amazon.whitewater.auxproxy.pbuffer.rs"
    ));
}

pub struct AuxProxyMessageSender {
    socket: futures_util::stream::SplitSink<
        WebSocketStream<MaybeTlsStream<tokio::net::TcpStream>>,
        tokio_tungstenite::tungstenite::Message,
    >,
}

impl AuxProxyMessageSender {
    pub fn new(
        socket: futures_util::stream::SplitSink<
            WebSocketStream<MaybeTlsStream<tokio::net::TcpStream>>,
            tokio_tungstenite::tungstenite::Message,
        >,
    ) -> Self {
        Self { socket }
    }

    pub async fn process_ready(&mut self, port: i32, log_paths_to_upload: Vec<String>) {
        let mut ready = sdk::ProcessReady::default();
        ready.port = port;
        ready.log_paths_to_upload = log_paths_to_upload;

        self.socket
            .send(tokio_tungstenite::tungstenite::Message::text("qwreyt"))
            .await;
    }
}

/*public ProcessReady(port: number, logPathsToUpload: string[]): Promise<GenericOutcome> {
const pReady = new sdk.com.amazon.whitewater.auxproxy.pbuffer.ProcessReady()
pReady.port = port
pReady.logPathsToUpload = logPathsToUpload

return this.EmitEventGeneric(pReady)
}*/
