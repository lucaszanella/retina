export CARGO_FFMPEG_MULTI_ARCHITECTURE_PREBUILT=/home/dev/orwell/deps/ffmpeg/build/
export CARGO_FEATURE_STATIC TRUE
cargo build --verbose -vv --target aarch64-linux-android --release --no-default-features --features "avcodec avfilter avformat swresample swscale"
#cargo build --verbose
