extern crate prost_build;

fn main() {
    println!("cargo:rerun-if-changed=src/sdk.proto");
    let mut config = prost_build::Config::new();
    config.type_attribute(".", "#[derive(serde::Serialize)]");
    config.type_attribute(".", "#[serde(rename_all = \"camelCase\")]");
    config
        .compile_protos(&["src/sdk.proto"], &["src/"])
        .unwrap();
}
