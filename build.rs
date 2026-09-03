fn main() {
    // compiles proto/ravenna.proto into rust code at build time
    prost_build::compile_protos(&["proto/ravenna.proto"], &["proto/"]).unwrap();
}
