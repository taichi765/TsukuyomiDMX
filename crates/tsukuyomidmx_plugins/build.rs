fn main() {
    tonic_prost_build::configure()
        .build_server(false)
        .compile_protos(
            &["ola/common/protocol/Ola.proto", "ola/common/rpc/Rpc.proto"],
            &["ola/"],
        )
        .unwrap();
}
