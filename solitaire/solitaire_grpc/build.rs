use std::env;

const PROTOC_PATH: &str = "protoc-21.2-win64/bin";

fn main() -> std::io::Result<()> {
    let root = env::var("CARGO_MANIFEST_DIR").unwrap();

    let path = match env::var("PATH") {
        Ok(v) => format!("{}\\{};{}", root, PROTOC_PATH, v),
        _ => format!("{}\\{}", root, PROTOC_PATH),
    };
    env::set_var("PATH", path);

    let proto_file = "./proto/solitaire.proto";

    tonic_build::compile_protos(proto_file)
}
