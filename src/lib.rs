pub mod aux_proxy_message_sender;
pub mod entity;
pub mod error;
pub mod log_parameters;
pub mod process_parameters;
pub mod server_state;

pub mod sdk {
    include!(concat!(
        env!("OUT_DIR"),
        "/com.amazon.whitewater.auxproxy.pbuffer.rs"
    ));
}

pub fn test_protobuf() -> sdk::ProcessEnding {
    sdk::ProcessEnding::default()
}
