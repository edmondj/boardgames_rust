use std::env;

const PROTOC_PATH: &str = "protoc-21.2-win64/bin";

fn main() {
    let root = env::var("CARGO_MANIFEST_DIR").unwrap();

    match env::var("PATH") {
        Ok(v) => env::set_var("PATH", format!("{}\\{};{}", root, PROTOC_PATH, v)),
        _ => env::set_var("PATH", format!("{}\\{}", root, PROTOC_PATH)),
    };

    let proto_file = "./proto/solitaire.proto";

    tonic_build::configure()
        .build_client(false)
        .build_server(true)
        .out_dir("./src/proto")
        .compile(&[proto_file], &["."])
        .unwrap_or_else(|e| panic!("protobuf compile error: {}", e));
}
