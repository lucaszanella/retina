fn main() {
    println!("cargo:rerun-if-changed=build.rs");
    println!("cargo:rustc-link-search=/usr/lib/x86_64-linux-gnu/");
    println!("cargo:rustc-link-lib=static=z");
    println!("cargo:rustc-link-lib=dylib=z");
    println!("cargo:rustc-link-lib=dylib=dl");
    println!("cargo:rustc-link-lib=static=X11");
    println!("cargo:rustc-link-lib=dylib=X11");
    println!("cargo:rustc-link-lib=dylib=Xext");
    println!("cargo:rustc-link-lib=dylib=vdpau");
    println!("cargo:rustc-link-lib=dylib=va");
    println!("cargo:rustc-link-lib=dylib=va-drm");
    println!("cargo:rustc-link-lib=dylib=va-x11");
    println!("cargo:rustc-link-lib=dylib=xcb");
}