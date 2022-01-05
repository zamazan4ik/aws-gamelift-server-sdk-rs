use aws_gamelift_server_sdk_rs::{
    log_parameters::LogParameters, process_parameters::ProcessParameters,
};

static CLIENT: once_cell::sync::Lazy<tokio::sync::Mutex<aws_gamelift_server_sdk_rs::api::Api>> =
    once_cell::sync::Lazy::new(|| {
        tokio::sync::Mutex::new(aws_gamelift_server_sdk_rs::api::Api::default())
    });

#[tokio::main]
async fn main() {
    env_logger::init();

    log::debug!(
        "AWS GameLift Server SDK version: {}",
        aws_gamelift_server_sdk_rs::api::Api::get_sdk_version()
    );

    if let Err(error) = CLIENT.lock().await.init_sdk().await {
        log::error!("{:?}", error);
    }

    let tokio_handle = tokio::runtime::Handle::current();
    if let Err(error) = CLIENT
        .lock()
        .await
        .process_ready(ProcessParameters {
            on_start_game_session: Box::new(move |game_session| {
                log::debug!("{:?}", game_session);

                tokio_handle.spawn(async {
                    CLIENT
                        .lock()
                        .await
                        .activate_game_session()
                        .await
                        .expect("Cannot activate game session");
                });
            }),
            on_update_game_session: Box::new(|update_game_session| {
                log::debug!("{:?}", update_game_session)
            }),
            on_process_terminate: Box::new(|| ()),
            on_health_check: Box::new(|| true),
            port: 14000,
            log_parameters: LogParameters { log_paths: vec!["test".to_string()] },
        })
        .await
    {
        log::error!("{:?}", error);
    }

    tokio::spawn(async move {
        loop {
            log::info!("Some game activity");
            tokio::time::sleep(std::time::Duration::from_secs(10)).await;
        }
    })
    .await
    .expect("Tokio task error");
}
