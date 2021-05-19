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
