use std::{
    collections::HashMap,
    time::{SystemTime, UNIX_EPOCH},
};

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Deserialize, Serialize)]
pub enum AttributeValue {
    S(String),
    D(f64),
    SL(Vec<String>),
    SDM(HashMap<String, f64>),
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum GameSessionStatus {
    #[default]
    NotSet,
    Active,
    Activating,
    Terminated,
    Terminating,
}

// todo: MatchmakerData

#[derive(Debug, Default, Clone, Serialize, Deserialize)]
#[serde(rename_all = "PascalCase", default)]
pub struct Player {
    pub player_id: String,
    pub player_attributes: HashMap<String, AttributeValue>,
    pub team: String,
    pub latency_in_ms: HashMap<String, i32>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "PascalCase", default)]
pub struct PlayerSession {
    pub player_id: String,
    pub player_session_id: String,
    pub game_session_id: String,
    pub fleet_id: String,
    pub ip_address: String,
    pub player_data: String,
    pub port: u16,
    #[serde(with = "crate::model::protocol::unix_time")]
    pub creation_time: SystemTime,
    #[serde(with = "crate::model::protocol::unix_time")]
    pub termination_time: SystemTime,
    pub status: PlayerSessionStatus,
    pub dns_name: String,
}

impl Default for PlayerSession {
    fn default() -> Self {
        Self {
            player_id: String::default(),
            player_session_id: String::default(),
            game_session_id: String::default(),
            fleet_id: String::default(),
            ip_address: String::default(),
            player_data: String::default(),
            port: u16::default(),
            creation_time: UNIX_EPOCH,
            termination_time: UNIX_EPOCH,
            status: PlayerSessionStatus::default(),
            dns_name: String::default(),
        }
    }
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum PlayerSessionCreationPolicy {
    #[default]
    NotSet,
    AcceptAll,
    DenyAll,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum PlayerSessionStatus {
    #[default]
    NotSet,
    Reserved,
    Active,
    Completed,
    Timedout,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum UpdateReason {
    MatchmakingDataUpdated,
    BackfillFailed,
    BackfillTimedOut,
    BackfillCancelled,
    #[default]
    Unknown,
}
