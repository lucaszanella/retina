use std::env;

fn main() {
    for (name, _value) in env::vars() {
        if name.starts_with("DEP_FFMPEG_") {
            println!(
                r#"cargo:rustc-cfg=feature="{}""#,
                name["DEP_FFMPEG_".len()..name.len()].to_lowercase()
            );
        }
    }
    //println!("cargo:rustc-link-lib=dylib=X11");/usr/include/x86_64-linux-gnu/libswscale/swscale.h

}
