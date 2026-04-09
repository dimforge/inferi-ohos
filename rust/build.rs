fn main() {
    napi_build_ohos::setup();
    // Required for OpenHarmony's musl-based libc (avoids __emutls_get_address errors)
    println!("cargo:rustc-link-lib=c++");
}
