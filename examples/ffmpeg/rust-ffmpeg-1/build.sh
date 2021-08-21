export CARGO_FFMPEG_MULTI_ARCHITECTURE_PREBUILT=/home/dev/orwell/deps/ffmpeg/build/
export CARGO_FEATURE_STATIC TRUE
cargo build --verbose -vv --release --no-default-features --features "codec device software-scaling"
