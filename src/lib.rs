pub mod api;
pub mod entity;
pub mod error;
mod http_client;
pub mod log_parameters;
mod mapper;
pub mod process_parameters;
pub mod protos;
pub mod server_state;
mod web_socket_listener;

/*mod sdk {
    include!(concat!(
        env!("OUT_DIR"),
        "/com.amazon.whitewater.auxproxy.pbuffer.rs"
    ));
}*/
