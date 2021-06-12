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
pub type NextToken = String;

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
            update_reason: UpdateReason::Unknown,
            backfill_ticket_id: "".to_string(),
        }
    }
}

#[derive(strum_macros::EnumString)]
pub enum UpdateReason {
    MatchmakingDataUpdated,
    BackfillFailed,
    BackfillTimedOut,
    BackfillCancelled,
    Unknown,
}

pub struct Player {
    pub player_id: Option<PlayerId>,
    pub player_attributes: Option<std::collections::HashMap<String, AttributeValue>>,
    pub team: Option<String>,
    pub latency_in_ms: Option<std::collections::HashMap<String, i32>>,
}

#[derive(strum_macros::ToString)]
pub enum PlayerSessionCreationPolicy {
    NotSet,
    AcceptAll,
    DenyAll,
}

/// This data type is used to specify which player session(s) to retrieve. It
/// can be used in several ways: (1) provide a PlayerSessionId to request a
/// specific player session; (2) provide a GameSessionId to request all player
/// sessions in the specified game session; or (3) provide a PlayerId to request
/// all player sessions for the specified player. For large collections of
/// player sessions, use the pagination parameters to retrieve results as
/// sequential pages.
pub struct DescribePlayerSessionsRequest {
    /// Unique game session identifier. Use this parameter to request all player
    /// sessions for the specified game session. Game session ID format is as
    /// follows: arn:aws:gamelift:<region>::gamesession/fleet-<fleet ID>/<ID
    /// string>. The value of <ID string> is either a custom ID string (if one
    /// was specified when the game session was created) a generated string.
    pub game_session_id: Option<GameSessionId>,

    /// Unique identifier for a player. Player IDs are defined by the developer.
    /// See Generate Player IDs.
    pub player_id: Option<PlayerId>,

    /// Unique identifier for a player session.
    pub player_session_id: Option<PlayerSessionId>,

    /// Player session status to filter results on. Possible player session
    /// statuses include the following:
    ///
    /// RESERVED – The player session request has been received, but the player
    /// has not yet connected to the server process and/or been validated.
    ///
    /// ACTIVE – The player has been validated by the server process and is
    /// currently connected.
    ///
    /// COMPLETED – The player connection has been dropped.
    ///
    /// TIMEDOUT – A player session request was received, but the player did not
    /// connect and/or was not validated within the time-out limit (60 seconds).
    pub player_session_status_filter: Option<String>,

    /// Token indicating the start of the next sequential page of results. Use
    /// the token that is returned with a previous call to this action. To
    /// specify the start of the result set, do not specify a value. If a player
    /// session ID is specified, this parameter is ignored.
    pub next_token: Option<NextToken>,

    /// Maximum number of results to return. Use this parameter with
    /// [next_token](self::next_token) to get results as a set of sequential
    /// pages. If a player session ID is specified, this parameter is
    /// ignored.
    pub limit: i32,
}

pub struct DescribePlayerSessionsResult {
    pub player_sessions: Vec<PlayerSession>,
    pub next_token: NextToken,
}

impl Default for DescribePlayerSessionsResult {
    fn default() -> Self {
        Self { player_sessions: vec![], next_token: "".to_string() }
    }
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

#[derive(strum_macros::EnumString)]
pub enum PlayerSessionStatus {
    NotSet,
    Reserved,
    Active,
    Completed,
    Timedout,
}

/// This data type is used to send a matchmaking backfill request. The
/// information is communicated to the GameLift service in a
/// [start_match_backfill](crate::api::Api::start_match_backfill) call.
pub struct StartMatchBackfillRequest {
    /// Unique identifier for a matchmaking or match backfill request ticket. If
    /// no value is provided here, Amazon GameLift will generate one in the form
    /// of a UUID. Use this identifier to track the match backfill ticket status
    /// or cancel the request if needed.
    pub ticket_id: Option<TicketId>,

    /// Unique game session identifier. The API action
    /// [GetGameSessionId](crate::api::Api::get_game_session_id) returns the
    /// identifier in ARN format.
    pub game_session_arn: Option<GameSessionArn>,

    /// Unique identifier, in the form of an ARN, for the matchmaker to use for
    /// this request. To find the matchmaker that was used to create the
    /// original game session, look in the game session object, in the
    /// matchmaker data property. Learn more about matchmaker data in Work with
    /// matchmaker data.
    pub matchmaking_configuration_arn: Option<MatchmakingConfigurationArn>,

    /// A set of data representing all players who are currently in the game
    /// session. The matchmaker uses this information to search for new players
    /// who are good matches for the current players. See the Amazon GameLift
    /// API Reference Guide for a description of the Player object format. To
    /// find player attributes, IDs, and team assignments, look in the game
    /// session object, in the matchmaker data property. If latency is used by
    /// the matchmaker, gather updated latency for the current region and
    /// include it in each player's data.
    pub players: Option<Vec<Player>>,
}

pub struct StartMatchBackfillResult {
    pub ticket_id: TicketId,
}

/// This data type is used to cancel a matchmaking backfill request. The
/// information is communicated to the GameLift service in a
/// [stop_match_backfill](crate::api::Api::stop_match_backfill) call.
pub struct StopMatchBackfillRequest {
    /// Unique identifier of the backfill request ticket to be canceled.
    pub ticket_id: Option<TicketId>,

    /// Unique game session identifier associated with the request being
    /// canceled.
    pub game_session_arn: Option<GameSessionArn>,

    /// Unique identifier of the matchmaker this request was sent to.
    pub matchmaking_configuration_arn: Option<MatchmakingConfigurationArn>,
}

pub struct AttributeValue {
    pub attr_type: AttrType,
    pub s: Option<String>,
    pub n: Option<f64>,
    pub sl: Option<Vec<String>>,
    pub sdm: Option<std::collections::HashMap<String, f64>>,
}

#[derive(Clone, Copy)]
pub enum AttrType {
    String = 1,
    Double,
    StringList,
    StringDoubleMap,
}

pub struct GetInstanceCertificateResult {
    pub certificate_path: String,
    pub private_key_path: String,
    pub certificate_chain_path: String,
    pub hostname: String,
    pub root_certificate_path: String,
}
