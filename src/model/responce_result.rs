use serde::Deserialize;

use crate::entity::PlayerSession;

const MAX_PLAYER_SESSIONS: usize = 1024;

#[derive(Debug, Default, Clone, Deserialize)]
#[serde(rename_all = "PascalCase", default)]
pub struct DescribePlayerSessionsResult {
    pub next_token: String,
    pub player_sessions: Vec<PlayerSession>,
}

impl DescribePlayerSessionsResult {
    pub fn add_player_session(&mut self, value: PlayerSession) {
        if self.player_sessions.len() < MAX_PLAYER_SESSIONS {
            self.player_sessions.push(value);
        } else {
            log::debug!("PlayerSessions count is greater than or equal to max player sessions {MAX_PLAYER_SESSIONS}.");
        }
    }
}

#[derive(Debug, Default, Clone, Deserialize)]
#[serde(rename_all = "PascalCase", default)]
pub struct GetComputeCertificateResult {
    pub certificate_path: String,
    pub compute_name: String,
}

#[derive(Debug, Default, Clone, Deserialize)]
#[serde(rename_all = "PascalCase", default)]
pub struct GetFleetRoleCredentialsResult {
    pub assumed_role_user_arn: String,
    pub assumed_role_id: String,
    pub access_key_id: String,
    pub secret_access_key: String,
    pub session_token: String,
    pub expiration: i64,
}

#[derive(Debug, Default, Clone, Deserialize)]
#[serde(rename_all = "PascalCase", default)]
pub struct StartMatchBackfillResult {
    pub ticket_id: String,
}
