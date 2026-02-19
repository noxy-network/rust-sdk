fn main() -> Result<(), Box<dyn std::error::Error>> {
    let protoc = protoc_bin_vendored::protoc_bin_path().map_err(|e| format!("protoc: {}", e))?;
    std::env::set_var("PROTOC", protoc);
    tonic_build::configure()
        .build_server(false)
        .build_client(true)
        .compile_protos(&["proto/noxy.proto"], &["proto"])?;
    Ok(())
}
