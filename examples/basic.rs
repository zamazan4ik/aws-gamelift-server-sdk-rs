fn main() {
    let _ = aws_gamelift_server_sdk_rs::api::Api::new();
    println!(
        "AWS GameLift Server SDK version: {}",
        aws_gamelift_server_sdk_rs::api::Api::get_sdk_version()
    );
}
