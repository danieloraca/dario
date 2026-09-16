fn main() {
    println!("cargo:rerun-if-changed=build.rs");

    if std::env::var("TARGET").as_deref() == Ok("wasm32-unknown-unknown") {
        // quad-snd's audio functions are provided by the bundled JavaScript.
        // Request imports explicitly instead of relying on rust-lld defaults.
        println!("cargo:rustc-link-arg-bin=dario=--import-undefined");
    }
}
