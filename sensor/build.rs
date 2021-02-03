extern crate protoc_rust;

fn main() {
    protoc_rust::Codegen::new()
        .out_dir("src/protos")
        .inputs(&["../shared/protos/sensor.proto"])
        .include("../shared/protos")
        .run()
        .expect("protoc");
}
