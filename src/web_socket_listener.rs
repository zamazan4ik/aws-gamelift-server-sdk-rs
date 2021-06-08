use futures_util::StreamExt;

const HOSTNAME: &'static str = "127.0.0.1";
const PORT: i32 = 5757;
const PID_KEY: &'static str = "pID";
const SDK_VERSION_KEY: &'static str = "sdkVersion";
const FLAVOR_KEY: &'static str = "sdkLanguage";
const FLAVOR: &'static str = "Rust";
const HEALTHCHECK_TIMEOUT_SECONDS: i32 = 60;

pub struct WebSocketListener {}

impl WebSocketListener {
    pub fn disconnect() {}

    pub async fn connect(&self) {}

    async fn perform_connect(&self) {
        let (mut ws_stream, _) = tokio_tungstenite::connect_async(Self::create_uri())
            .await
            .expect("Failed to connect");

        while let Some(msg) = ws_stream.next().await {
            let msg = msg.unwrap();
            if msg.is_text() {
                let message_text = msg.into_text().unwrap();
            } else if msg.is_close() {
                log::debug!("Socket disconnected. Message: {}", msg);
            }
        }
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
