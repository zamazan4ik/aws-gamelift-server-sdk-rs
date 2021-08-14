fn main() {
    let out_dir = std::env::var("OUT_DIR").expect("OUT_DIR env variable not found");
    let generated_with_pure_dir = format!("{}/generated_with_pure", out_dir);
    if std::path::Path::new(&generated_with_pure_dir).exists() {
        std::fs::remove_dir_all(&generated_with_pure_dir)
            .unwrap_or_else(|_| panic!("Cannot remove the directory: {}", generated_with_pure_dir));
    }
    std::fs::create_dir(&generated_with_pure_dir)
        .unwrap_or_else(|_| panic!("Cannot create the directory: {}", generated_with_pure_dir));
    protobuf_codegen_pure::Codegen::new()
        .customize(protobuf_codegen_pure::Customize {
            serde_derive: Some(true),
            gen_mod_rs: Some(true),
            ..Default::default()
        })
        .out_dir(generated_with_pure_dir)
        .input("src/protos/sdk.proto")
        .include("src/protos")
        .run()
        .expect("Protobuf codegen failed.");

    println!("cargo:rerun-if-changed=src/protos/sdk.proto");
}
