use aws_gamelift_server_sdk_rs::*;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    env_logger::builder().filter_level(log::LevelFilter::Info).init();
    log::info!("AWS GameLift Server SDK version: {}", Api::get_sdk_version());

    let web_socket_url = "wss://ap-northeast-1.api.amazongamelift.com";
    let compute_name = "MyComputeName";
    let fleet_id = "fleet-xxxxxxxx-xxxx-xxxx-xxxx-xxxxxxxxxxxx";
    let port: u16 = 7777; // This example is hard-coded for simplicity
    let log_parameters = LogParameters::new(["test"]);

    let shared_config = aws_config::load_from_env().await;
    let client = aws_sdk_gamelift::Client::new(&shared_config);

    let compute_auth_token = client
        .get_compute_auth_token()
        .compute_name(compute_name)
        .fleet_id(fleet_id)
        .send()
        .await?;
    let auth_token = compute_auth_token.auth_token.expect("Need auth_token to connect");

    let server_parameters = ServerParameters::new(
        web_socket_url,
        uuid::Uuid::new_v4().to_string(),
        compute_name,
        fleet_id,
        auth_token,
    );

    // Alternatively, you can use the default if the server is hosted only on
    // GameLift managed EC2 instances. let server_parameters =
    // ServerParameters::default();

    let api = Api::init_sdk(server_parameters).await?;

    let (process_parameters, mut event_receiver) = bind_channel_on_callbacks(port, log_parameters);
    api.process_ready(process_parameters).await?;

    loop {
        tokio::select! {
            _ = tokio::signal::ctrl_c() => break,
            Some(event) = event_receiver.recv() => {
                match event {
                    ServerEvent::OnStartGameSession(game_session) => {
                        log::info!("{:?}", game_session);

                        api.activate_game_session().await?;

                        log::info!("Session active!");
                    }
                    ServerEvent::OnUpdateGameSession(update_game_session) => {
                        log::info!("{:?}", update_game_session)
                    }
                    ServerEvent::OnProcessTerminate() => {
                        break;
                    }
                    ServerEvent::OnHealthCheck(feedback) => {
                        let _ = feedback.send(true);
                    }
                }
            }
            else => break,
        }
    }

    api.process_ending().await?;
    Ok(())
}
