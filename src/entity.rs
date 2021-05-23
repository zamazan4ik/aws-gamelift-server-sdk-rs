pub struct GameProperty {
    pub key: Option<String>,
    pub value: Option<String>,
}

pub type GameSessionId = String;
pub type PlayerId = String;
pub type PlayerSessionId = String;
pub type TerminationTimeType = i64;
pub type FleetId = String;
pub type TicketId = String;
pub type GameSessionArn = String;
pub type MatchmakingConfigurationArn = String;

pub struct GameSession {
    pub game_session_id: Option<GameSessionId>,
    pub name: Option<String>,
    pub fleet_id: Option<String>,
    pub max_player_session_count: i32,
    pub port: i32,
    pub ip_address: Option<String>,
    pub game_session_data: Option<String>,
    pub matchmaker_data: Option<String>,
    pub game_properties: Vec<GameProperty>,
    pub dns_name: Option<String>,
}

impl Default for GameSession {
    fn default() -> Self {
        Self {
            game_session_id: None,
            name: None,
            fleet_id: None,
            max_player_session_count: 0,
            port: 0,
            ip_address: None,
            game_session_data: None,
            matchmaker_data: None,
            game_properties: vec![],
            dns_name: None,
        }
    }
}

pub struct UpdateGameSession {
    pub game_session: Option<GameSession>,
    pub update_reason: UpdateReason,
    pub backfill_ticket_id: String,
}

impl Default for UpdateGameSession {
    fn default() -> Self {
        Self {
            game_session: Default::default(),
            update_reason: UpdateReason::UNKNOWN,
            backfill_ticket_id: "".to_string(),
        }
    }
}

#[derive(strum_macros::EnumString)]
pub enum UpdateReason {
    MATCHMAKING_DATA_UPDATED,
    BACKFILL_FAILED,
    BACKFILL_TIMED_OUT,
    BACKFILL_CANCELLED,
    UNKNOWN,
}

#[derive(strum_macros::ToString)]
pub enum PlayerSessionCreationPolicy {
    NotSet,
    AcceptAll,
    DenyAll,
}

pub struct DescribePlayerSessionsRequest {
    pub game_session_id: Option<GameSessionId>,
    pub player_id: Option<PlayerId>,
    pub player_session_id: Option<PlayerSessionId>,
    pub player_session_status_filter: Option<String>,
    pub next_token: Option<String>,
    pub limit: i32,
}

pub struct PlayerSession {
    pub player_id: Option<PlayerId>,
    pub player_session_id: Option<PlayerSessionId>,
    pub game_session_id: Option<GameSessionId>,
    pub fleet_id: Option<FleetId>,
    pub ip_address: Option<String>,
    pub player_data: Option<String>,
    pub port: i32,
    pub creation_time: i64,
    pub termination_time: i64,
    pub status: PlayerSessionStatus,
    pub dns_name: Option<String>,
}

pub enum PlayerSessionStatus {
    NotSet,
    RESERVED,
    ACTIVE,
    COMPLETED,
    TIMEDOUT,
}

pub struct StopMatchBackfillRequest {
    pub ticket_id: Option<TicketId>,
    pub game_session_arn: Option<GameSessionArn>,
    pub matchmaking_configuration_arn: Option<MatchmakingConfigurationArn>,
}
