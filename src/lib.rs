pub mod api;
pub mod aux_proxy_message_sender;
pub mod entity;
pub mod error;
pub mod log_parameters;
mod mapper;
pub mod network;
pub mod process_parameters;
pub mod server_state;

mod sdk {
    include!(concat!(
        env!("OUT_DIR"),
        "/com.amazon.whitewater.auxproxy.pbuffer.rs"
    ));
}
