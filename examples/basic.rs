use std::sync::Arc;

use aws_gamelift_server_sdk_rs::{Api, LogParameters, ProcessParameters, ServerParameters};

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

    // Alternatively, you can use the default if the server is hosted only on GameLift managed EC2 instances.
    // let server_parameters = ServerParameters::default();

    let api = Arc::new(Api::init_sdk(server_parameters).await?);

    api.process_ready(ProcessParameters {
        port,
        log_parameters,
        on_start_game_session: {
            let api = api.clone();
            Box::new(move |game_session| {
                let api = api.clone();
                Box::pin(async move {
                    log::info!("{:?}", game_session);

                    api.activate_game_session().await.expect("Cannot activate game session");

                    log::info!("Session active!");
                })
            })
        },
        on_update_game_session: Box::new(|update_game_session| {
            Box::pin(async move { log::info!("{:?}", update_game_session) })
        }),
        on_process_terminate: Box::new(|| Box::pin(async {})),
        on_health_check: Box::new(|| Box::pin(async { true })),
    })
    .await?;

    loop {
        log::info!("Some game activity");
        tokio::signal::ctrl_c().await?;
        break;
    }

    Ok(())
}
