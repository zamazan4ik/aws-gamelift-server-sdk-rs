pub type OnStartGameSessionType = dyn Fn(crate::entity::GameSession) + std::marker::Send;
pub type OnUpdateGameSessionType = dyn Fn(crate::entity::UpdateGameSession) + std::marker::Send;
pub type OnProcessTerminateType = dyn Fn() + std::marker::Send;
pub type OnHealthCheckType = dyn Fn() -> bool + std::marker::Send;

pub struct ProcessParameters {
    pub on_start_game_session: Box<OnStartGameSessionType>,
    pub on_update_game_session: Box<OnUpdateGameSessionType>,
    pub on_process_terminate: Box<OnProcessTerminateType>,
    pub on_health_check: Box<OnHealthCheckType>,
    pub port: i32,
    pub log_parameters: crate::log_parameters::LogParameters,
}
