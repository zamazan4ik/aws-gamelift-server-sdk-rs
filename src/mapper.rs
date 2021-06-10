use protobuf::RepeatedField;

pub fn game_session_mapper(
    source_game_session: crate::protos::generated_with_pure::sdk::GameSession,
) -> crate::entity::GameSession {
    let mut converted_game_session = crate::entity::GameSession::default();
    converted_game_session.game_session_id = Some(source_game_session.gameSessionId);
    converted_game_session.name = Some(source_game_session.name);
    converted_game_session.fleet_id = Some(source_game_session.fleetId);
    converted_game_session.max_player_session_count = source_game_session.maxPlayers;
    converted_game_session.port = source_game_session.port;
    converted_game_session.ip_address = Some(source_game_session.ipAddress);
    converted_game_session.game_session_data = Some(source_game_session.gameSessionData);
    converted_game_session.matchmaker_data = Some(source_game_session.matchmakerData);
    converted_game_session.dns_name = Some(source_game_session.dnsName);

    for game_property in source_game_session.gameProperties {
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
    source: crate::protos::generated_with_pure::sdk::UpdateGameSession,
) -> crate::entity::UpdateGameSession {
    let mut converted_update_game_session = crate::entity::UpdateGameSession::default();

    converted_update_game_session.game_session =
        Some(game_session_mapper(source.gameSession.unwrap()));

    use std::str::FromStr;
    converted_update_game_session.update_reason =
        crate::entity::UpdateReason::from_str(&source.updateReason).unwrap();
    converted_update_game_session.backfill_ticket_id = source.backfillTicketId;

    converted_update_game_session
}

pub fn describe_player_session_request_mapper(
    source: crate::protos::generated_with_pure::sdk::DescribePlayerSessionsResponse,
) -> crate::entity::DescribePlayerSessionsResult {
    let mut result = crate::entity::DescribePlayerSessionsResult::default();

    result.next_token = source.nextToken;

    use std::str::FromStr;
    for player_session in source.playerSessions {
        let converted_player_session = crate::entity::PlayerSession {
            player_id: Some(player_session.playerId),
            player_session_id: Some(player_session.playerSessionId),
            game_session_id: Some(player_session.gameSessionId),
            fleet_id: Some(player_session.fleetId),
            ip_address: Some(player_session.ipAddress),
            player_data: Some(player_session.playerData),
            port: player_session.port,
            creation_time: player_session.creationTime,
            termination_time: player_session.terminationTime,
            status: crate::entity::PlayerSessionStatus::from_str(&player_session.status).unwrap(),
            dns_name: Some(player_session.dnsName),
        };

        result.player_sessions.push(converted_player_session);
    }

    result
}

pub fn start_matchmaking_result_mapper(
    source: crate::protos::generated_with_pure::sdk::BackfillMatchmakingResponse,
) -> crate::entity::StartMatchBackfillResult {
    crate::entity::StartMatchBackfillResult {
        ticket_id: source.ticketId,
    }
}

pub fn get_instance_certificate_result_mapper(
    source: crate::protos::generated_with_pure::sdk::GetInstanceCertificateResponse,
) -> crate::entity::GetInstanceCertificateResult {
    crate::entity::GetInstanceCertificateResult {
        certificate_path: source.certificatePath,
        private_key_path: source.privateKeyPath,
        certificate_chain_path: source.certificateChainPath,
        hostname: source.hostName,
        root_certificate_path: source.rootCertificatePath,
    }
}

pub fn stop_matchmaking_request_mapper(
    source: crate::entity::StopMatchBackfillRequest,
) -> crate::protos::generated_with_pure::sdk::StopMatchmakingRequest {
    let mut result = crate::protos::generated_with_pure::sdk::StopMatchmakingRequest::default();
    result.ticketId = source.ticket_id.unwrap();
    result.gameSessionArn = source.game_session_arn.unwrap();
    result.matchmakingConfigurationArn = source.matchmaking_configuration_arn.unwrap();

    result
}

pub fn attribute_value_mapper(
    source: crate::entity::AttributeValue,
) -> crate::protos::generated_with_pure::sdk::AttributeValue {
    let mut result = crate::protos::generated_with_pure::sdk::AttributeValue::default();
    result.field_type = source.attr_type as i32;
    match source.attr_type {
        crate::entity::AttrType::STRING => {
            result.S = source.s.unwrap();
        }
        crate::entity::AttrType::DOUBLE => {
            result.N = source.n.unwrap();
        }
        crate::entity::AttrType::STRING_LIST => {
            result.SL = RepeatedField::from_vec(source.sl.unwrap());
        }
        crate::entity::AttrType::STRING_DOUBLE_MAP => {
            result.SDM = source.sdm.unwrap();
        }
    }

    result
}

pub fn player_mapper(
    source: crate::entity::Player,
) -> crate::protos::generated_with_pure::sdk::Player {
    let mut result = crate::protos::generated_with_pure::sdk::Player::default();
    result.playerId = source.player_id.unwrap();
    result.team = source.team.unwrap();

    if let Some(latency_in_ms) = source.latency_in_ms {
        result.latencyInMs = latency_in_ms;
    }

    if let Some(player_attributes) = source.player_attributes {
        for (id, player_attribute) in player_attributes {
            result
                .playerAttributes
                .insert(id, attribute_value_mapper(player_attribute));
        }
    }

    result
}

pub fn start_match_backfill_request_mapper(
    source: crate::entity::StartMatchBackfillRequest,
) -> crate::protos::generated_with_pure::sdk::BackfillMatchmakingRequest {
    let mut result = crate::protos::generated_with_pure::sdk::BackfillMatchmakingRequest::default();
    result.ticketId = source.ticket_id.unwrap();
    result.gameSessionArn = source.game_session_arn.unwrap();
    result.matchmakingConfigurationArn = source.matchmaking_configuration_arn.unwrap();

    if let Some(players) = source.players {
        for player in players {
            result.players.push(player_mapper(player));
        }
    }

    result
}

pub fn describe_player_sessions_mapper(
    source: crate::entity::DescribePlayerSessionsRequest,
) -> crate::protos::generated_with_pure::sdk::DescribePlayerSessionsRequest {
    let mut result =
        crate::protos::generated_with_pure::sdk::DescribePlayerSessionsRequest::default();

    if let Some(game_session_id) = source.game_session_id {
        result.gameSessionId = game_session_id;
    }

    if let Some(player_id) = source.player_id {
        result.playerId = player_id;
    }

    if let Some(player_session_id) = source.player_session_id {
        result.playerSessionId = player_session_id;
    }

    if let Some(player_session_status_filter) = source.player_session_status_filter {
        result.playerSessionStatusFilter = player_session_status_filter;
    }

    if let Some(next_token) = source.next_token {
        result.nextToken = next_token;
    }

    result.limit = source.limit;

    result
}
