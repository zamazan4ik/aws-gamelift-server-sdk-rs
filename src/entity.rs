pub struct GameProperty {
    pub key: Option<String>,
    pub value: Option<String>,
}

pub type GameSessionId = String;

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
