use std::error::Error;

fn main() -> Result<(), Box<dyn Error>> {
    build_control()?;
    Ok(())
}

fn build_control() -> Result<(), Box<dyn Error>> {
    let iface_files = &[
        "./protos_test_proto3_optional/email.proto",
        "./protos_test_proto3_optional/templating.proto",
        "./protos_test_proto3_optional/user.proto",
        "./protos_test_proto3_optional/notification.proto",
    ];
    let dirs = &["./protos"];

    tonic_build::configure()
        .build_client(true)
        .build_server(true)
        .compile(iface_files, dirs)?;

    // recompile protobufs only if any of the proto files changes.
    for file in iface_files {
        println!("cargo:rerun-if-changed={}", file);
    }

    Ok(())
}
