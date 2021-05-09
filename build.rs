extern crate prost_build;

fn main() {
    println!("cargo:rerun-if-changed=src/sdk.proto");
    prost_build::compile_protos(&["src/sdk.proto"], &["src/"]).unwrap();
}
