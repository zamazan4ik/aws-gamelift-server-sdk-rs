pub mod error;

pub mod sdk {
    include!(concat!(
        env!("OUT_DIR"),
        "/com.amazon.whitewater.auxproxy.pbuffer.rs"
    ));
}

pub fn test_protobuf() -> sdk::ProcessEnding {
    sdk::ProcessEnding::default()
}
