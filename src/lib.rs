//! An unofficial port of AWS GameLift Server SDK for Rust.
//!
//! Official documentation for the SDK (C# version), can be found [here](https://docs.aws.amazon.com/gamelift/latest/developerguide/integration-server-sdk-csharp-ref.html).

pub mod api;
pub mod entity;
pub mod error;
mod http_client;
pub mod log_parameters;
mod mapper;
pub mod model;
pub mod process_parameters;
pub mod protos;
pub mod server_parameters;
pub mod server_state;
pub mod web_socket_listener;
