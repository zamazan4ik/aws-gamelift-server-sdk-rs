//! An unofficial port of AWS GameLift Server SDK for Rust.
//!
//! Official documentation for the SDK (C# version), can be found [here](https://docs.aws.amazon.com/gamelift/latest/developerguide/integration-server-sdk-csharp-ref.html).

mod api;
mod connection_state;
mod error;
mod log_parameters;
pub mod model;
mod process_parameters;
mod server_parameters;
mod server_state;
mod web_socket_listener;

pub use api::Api;
pub use error::GameLiftErrorType;
pub use log_parameters::LogParameters;
pub use process_parameters::{GameLiftEventCallbacks, ProcessParameters};
pub use server_parameters::ServerParameters;
