use serde::Serialize;

use super::{protocol::RequestContent, responce_result, Player, PlayerSessionCreationPolicy};

#[derive(Debug, Serialize)]
#[serde(rename_all = "PascalCase")]
pub struct AcceptPlayerSessionRequest {
    pub game_session_id: String,
    pub player_session_id: String,
}

impl RequestContent for AcceptPlayerSessionRequest {
    const ACTION_NAME: &'static str = "AcceptPlayerSession";
    type Response = ();
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "PascalCase")]
pub struct ActivateGameSessionRequest {
    pub game_session_id: String,
}

impl RequestContent for ActivateGameSessionRequest {
    const ACTION_NAME: &'static str = "ActivateGameSession";
    type Response = ();
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "PascalCase")]
pub struct ActivateServerProcessRequest {
    pub sdk_version: String,
    pub sdk_language: String,
    pub port: u16,
    pub log_paths: Vec<String>,
}

impl RequestContent for ActivateServerProcessRequest {
    const ACTION_NAME: &'static str = "ActivateServerProcess";
    type Response = ();
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "PascalCase")]
pub struct DescribePlayerSessionsRequest {
    pub game_session_id: String,
    pub player_session_id: String,
    pub player_id: String,
    pub player_session_status_filter: String,
    pub next_token: String,
    pub limit: i32,
}

impl RequestContent for DescribePlayerSessionsRequest {
    const ACTION_NAME: &'static str = "DescribePlayerSessions";
    type Response = responce_result::DescribePlayerSessionsResult;
}

impl Default for DescribePlayerSessionsRequest {
    fn default() -> Self {
        Self {
            game_session_id: String::default(),
            player_session_id: String::default(),
            player_id: String::default(),
            player_session_status_filter: String::default(),
            next_token: String::default(),
            limit: 50,
        }
    }
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "PascalCase")]
pub struct GetComputeCertificateRequest {}

impl RequestContent for GetComputeCertificateRequest {
    const ACTION_NAME: &'static str = "GetComputeCertificate";
    type Response = responce_result::GetComputeCertificateResult;
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "PascalCase")]
pub struct GetFleetRoleCredentialsRequest {
    pub role_arn: String,
    pub role_session_name: String,
}

impl RequestContent for GetFleetRoleCredentialsRequest {
    const ACTION_NAME: &'static str = "GetFleetRoleCredentials";
    type Response = responce_result::GetFleetRoleCredentialsResult;
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "PascalCase")]
pub struct HeartbeatServerProcessRequest {
    pub health_status: bool,
}

impl RequestContent for HeartbeatServerProcessRequest {
    const ACTION_NAME: &'static str = "HeartbeatServerProcess";
    type Response = ();
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "PascalCase")]
pub struct RemovePlayerSessionRequest {
    pub game_session_id: String,
    pub player_session_id: String,
}

impl RequestContent for RemovePlayerSessionRequest {
    const ACTION_NAME: &'static str = "RemovePlayerSession";
    type Response = ();
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "PascalCase")]
pub struct StartMatchBackfillRequest {
    pub game_session_arn: String,
    pub matchmaking_configuration_arn: String,
    pub players: Vec<Player>,
    pub ticket_id: String,
}

impl RequestContent for StartMatchBackfillRequest {
    const ACTION_NAME: &'static str = "StartMatchBackfill";
    type Response = responce_result::StartMatchBackfillResult;
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "PascalCase")]
pub struct StopMatchBackfillRequest {
    pub game_session_arn: String,
    pub matchmaking_configuration_arn: String,
    pub ticket_id: String,
}

impl RequestContent for StopMatchBackfillRequest {
    const ACTION_NAME: &'static str = "StopMatchBackfill";
    type Response = ();
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "PascalCase")]
pub struct TerminateServerProcessRequest {}

impl RequestContent for TerminateServerProcessRequest {
    const ACTION_NAME: &'static str = "TerminateServerProcess";
    type Response = ();
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "PascalCase")]
pub struct UpdatePlayerSessionCreationPolicyRequest {
    pub game_session_id: String,
    pub player_session_policy: PlayerSessionCreationPolicy,
}

impl RequestContent for UpdatePlayerSessionCreationPolicyRequest {
    const ACTION_NAME: &'static str = "UpdatePlayerSessionCreationPolicy";
    type Response = ();
}
