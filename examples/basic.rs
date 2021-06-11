use aws_gamelift_server_sdk_rs::{
    entity::*, log_parameters::LogParameters, process_parameters::ProcessParameters,
};

#[tokio::main]
async fn main() {
    env_logger::init();

    let mut client = aws_gamelift_server_sdk_rs::api::Api::new();
    log::debug!(
        "AWS GameLift Server SDK version: {}",
        aws_gamelift_server_sdk_rs::api::Api::get_sdk_version()
    );

    if let Err(error) = client.init_sdk().await {
        log::error!("{:?}", error);
    }

    if let Err(error) = client
        .process_ready(ProcessParameters {
            on_start_game_session: Box::new(|game_session| ()),
            on_update_game_session: Box::new(|update_game_session| ()),
            on_process_terminate: Box::new(|| ()),
            on_health_check: Box::new(|| true),
            port: 14000,
            log_parameters: LogParameters { log_paths: vec!["test".to_string()] },
        })
        .await
    {
        log::error!("{:?}", error);
    }

    tokio::spawn(async {
        loop {
            log::info!("Some game activity");
            tokio::time::sleep(std::time::Duration::from_secs(10)).await;
        }
    })
    .await;
}
