use aws_gamelift_server_sdk_rs::{
    log_parameters::LogParameters, process_parameters::ProcessParameters,
    server_parameters::ServerParameters,
};

#[tokio::main]
async fn main() {
    env_logger::init();

    log::debug!(
        "AWS GameLift Server SDK version: {}",
        aws_gamelift_server_sdk_rs::api::Api::get_sdk_version()
    );

    let server_parameters = ServerParameters::default();
    let api = match aws_gamelift_server_sdk_rs::api::Api::init_sdk(server_parameters).await {
        Ok(api) => std::sync::Arc::new(tokio::sync::Mutex::new(api)),
        Err(error) => {
            log::error!("{:?}", error);
            return;
        }
    };

    if let Err(error) = api
        .lock()
        .await
        .process_ready(ProcessParameters {
            on_start_game_session: {
                let api = api.clone();
                Box::new(move |game_session| {
                    let api = api.clone();
                    Box::pin(async move {
                        log::debug!("{:?}", game_session);

                        let lock = api.lock().await;
                        lock.activate_game_session().await.expect("Cannot activate game session");

                        log::info!("Session active!");
                    })
                })
            },
            on_update_game_session: Box::new(|update_game_session| {
                Box::pin(async move { log::debug!("{:?}", update_game_session) })
            }),
            on_process_terminate: Box::new(|| Box::pin(async {})),
            on_health_check: Box::new(|| Box::pin(async { true })),
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
