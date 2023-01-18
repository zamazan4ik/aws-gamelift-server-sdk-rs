/// Connection information and methods for maintaining the connection between GameLift
/// and your game server.
#[derive(Debug, Default)]
pub struct ServerParameters {
    pub web_socket_url: String,
    pub process_id: String,
    pub host_id: String,
    pub fleet_id: String,
    pub auth_token: String,
}

impl ServerParameters {
    pub fn new(
        web_socket_url: impl Into<String>,
        process_id: impl Into<String>,
        host_id: impl Into<String>,
        fleet_id: impl Into<String>,
        auth_token: impl Into<String>,
    ) -> Self {
        Self {
            web_socket_url: web_socket_url.into(),
            process_id: process_id.into(),
            host_id: host_id.into(),
            fleet_id: fleet_id.into(),
            auth_token: auth_token.into(),
        }
    }
}