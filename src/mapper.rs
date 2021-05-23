pub fn game_session_mapper(
    source_game_session: crate::sdk::GameSession,
) -> crate::entity::GameSession {
    let mut converted_game_session = crate::entity::GameSession::default();
    converted_game_session.game_session_id = Some(source_game_session.game_session_id);
    converted_game_session.name = Some(source_game_session.name);
    converted_game_session.fleet_id = Some(source_game_session.fleet_id);
    converted_game_session.max_player_session_count = source_game_session.max_players;
    converted_game_session.port = source_game_session.port;
    converted_game_session.ip_address = Some(source_game_session.ip_address);
    converted_game_session.game_session_data = Some(source_game_session.game_session_data);
    converted_game_session.matchmaker_data = Some(source_game_session.matchmaker_data);
    converted_game_session.dns_name = Some(source_game_session.dns_name);

    for game_property in source_game_session.game_properties {
        converted_game_session
            .game_properties
            .push(crate::entity::GameProperty {
                key: Some(game_property.key),
                value: Some(game_property.value),
            });
    }

    converted_game_session
}

pub fn update_game_session_mapper(
    source: crate::sdk::UpdateGameSession,
) -> crate::entity::UpdateGameSession {
    let mut converted_update_game_session = crate::entity::UpdateGameSession::default();

    converted_update_game_session.game_session =
        Some(game_session_mapper(source.game_session.unwrap()));

    use std::str::FromStr;
    converted_update_game_session.update_reason =
        crate::entity::UpdateReason::from_str(&source.update_reason).unwrap();
    converted_update_game_session.backfill_ticket_id = source.backfill_ticket_id;

    converted_update_game_session
}

pub fn stop_matchmaking_request_mapper(
    source: crate::entity::StopMatchBackfillRequest,
) -> crate::sdk::StopMatchmakingRequest {
    let mut result = crate::sdk::StopMatchmakingRequest::default();
    result.ticket_id = source.ticket_id.unwrap();
    result.game_session_arn = source.game_session_arn.unwrap();
    result.matchmaking_configuration_arn = source.matchmaking_configuration_arn.unwrap();

    result
}

/*public static CreateBufferedStopMatchmakingRequest(
request: StopMatchBackfillRequest
): sdk.com.amazon.whitewater.auxproxy.pbuffer.StopMatchmakingRequest {
const translated: sdk.com.amazon.whitewater.auxproxy.pbuffer.StopMatchmakingRequest = new sdk.com.amazon.whitewater.auxproxy.pbuffer.StopMatchmakingRequest()

translated.ticketId = request.TicketId!
translated.gameSessionArn = request.GameSessionArn!
translated.matchmakingConfigurationArn = request.MatchmakingConfigurationArn!

return translated
}*/
