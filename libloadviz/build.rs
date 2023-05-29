extern crate cbindgen;

use std::env;

// Inspired by: https://github.com/mozilla/cbindgen/blob/master/docs.md#buildrs
fn main() {
    let crate_dir = env::var("CARGO_MANIFEST_DIR").unwrap();

    cbindgen::Builder::new()
      .with_crate(crate_dir)
      .generate()
      .expect("Unable to generate bindings")
      .write_to_file("loadviz.h");
}
