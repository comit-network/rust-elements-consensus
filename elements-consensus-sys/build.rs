use std::path::PathBuf;

fn main() {
    // Build the project in the path `foo` and installs it in `$OUT_DIR`
    let dst = autotools::Config::new("depend/elements")
        .reconf("-ivf")
        .enable_static()
        .disable_shared()
        .enable("liquid", None)
        .disable("gui-tests", None)
        .disable("tests", None)
        .disable("bench", None)
        .disable("wallet", None)
        .disable("zmq", None)
        .disable("util-wallet", None)
        .disable("util-cli", None)
        .disable("util-tx", None)
        .without("gui", None)
        .without("daemon", None)
        .without("server", None)
        .without("upnp", None)
        .build();

    // Simply link the library without using pkg-config
    println!("cargo:rustc-link-search=native={}/lib", dst.display());
    println!("cargo:rustc-link-lib=static=elementsconsensus");

    let binding_file = out_dir().join("bindings.rs");

    bindgen::Builder::default()
        .header("depend/elements/src/script/bitcoinconsensus.h")
        .generate_comments(true)
        .generate()
        .expect("Unable to generate bindings")
        .write_to_file(&binding_file)
        .expect("Couldn't write bindings!");

    println!("cargo:rerun-if-changed=depend/elements-HEAD-revision.txt");
    println!("cargo:rerun-if-changed=build.rs");
}

fn out_dir() -> PathBuf {
    std::env::var("OUT_DIR")
        .expect("OUT_DIR environment var not set.")
        .into()
}
