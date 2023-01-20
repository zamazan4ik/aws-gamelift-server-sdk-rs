pub mod message;
mod protocol;
pub mod request;
pub mod responce_result;

pub use protocol::*;

pub type GameSession = message::CreateGameSessionMessage;
pub type UpdateGameSession = message::UpdateGameSessionMessage;
