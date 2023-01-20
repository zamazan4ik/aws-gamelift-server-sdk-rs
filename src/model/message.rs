use std::collections::HashMap;

use serde::Deserialize;

use super::GameSession;

#[derive(Debug, Default, Clone, Deserialize)]
#[serde(rename_all = "PascalCase", default)]
pub struct CreateGameSessionMessage {
    pub game_session_id: String,
    #[serde(rename = "GameSessionName")]
    pub name: String,
    // Field does not exist in the original
    pub fleet_id: String,
    pub maximum_player_session_count: i32,
    pub port: u16,
    pub ip_address: String,
    pub game_session_data: String,
    pub matchmaker_data: String,
    pub game_properties: HashMap<String, String>,
    pub dns_name: String,
}

impl CreateGameSessionMessage {
    pub const ACTION_NAME: &'static str = "CreateGameSession";
}

#[derive(Debug, Default, Clone, Deserialize)]
#[serde(rename_all = "PascalCase", default)]
pub struct RefreshConnectionMessage {
    pub refresh_connection_endpoint: String,
    pub auth_token: String,
}

impl RefreshConnectionMessage {
    pub const ACTION_NAME: &'static str = "RefreshConnection";
}

#[derive(Debug, Default, Clone, Deserialize)]
#[serde(rename_all = "PascalCase", default)]
pub struct TerminateProcessMessage {
    pub termination_time: i64,
}

impl TerminateProcessMessage {
    pub const ACTION_NAME: &'static str = "TerminateProcess";
}

#[derive(Debug, Default, Clone, Deserialize)]
#[serde(rename_all = "PascalCase", default)]
pub struct UpdateGameSessionMessage {
    pub game_session: GameSession,
    pub update_reason: crate::entity::UpdateReason,
    pub backfill_ticket_id: String,
}

impl UpdateGameSessionMessage {
    pub const ACTION_NAME: &'static str = "UpdateGameSession";
}
