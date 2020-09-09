fn main() {
    tonic_build::compile_protos("proto/pomodoro.proto").unwrap();
}
