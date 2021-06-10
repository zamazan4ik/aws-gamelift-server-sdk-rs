#[tokio::main]
async fn main() {
    let mut client = aws_gamelift_server_sdk_rs::api::Api::new();
    println!(
        "AWS GameLift Server SDK version: {}",
        aws_gamelift_server_sdk_rs::api::Api::get_sdk_version()
    );

    if let Err(error) = client.init_sdk().await {
        println!("{:?}", error);
    }
}
