use std::env;

fn main() {
    let crate_dir = env::var("CARGO_MANIFEST_DIR").unwrap();

    cbindgen::Builder::new()
        .with_crate(crate_dir)
        .with_language(cbindgen::Language::Cxx)
        .with_pragma_once(true)
        .with_namespace("uvc_control")
        .with_tab_width(4)
        .generate()
        .expect("Unable to generate bindings")
        .write_to_file("include/uvc_control.h");
}
