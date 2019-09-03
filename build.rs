extern crate gl_generator;

use gl_generator::{Registry, Api, Profile, Fallbacks, GlobalGenerator};
use std::env;
use std::fs::File;
use std::path::Path;

fn main() {
    println!(r"cargo:rustc-link-search=native=/usr/lib/x86_64-linux-gnu");
    let dest = env::var("OUT_DIR").unwrap();
    let mut file = File::create(&Path::new(&dest).join("bindings.rs")).unwrap();

    // Registry::new(Api::Gl, (4, 5), Profile::Core, Fallbacks::All, [])
    Registry::new(Api::Gl, (4, 5), Profile::Compatibility, Fallbacks::All, [])
        .write_bindings(GlobalGenerator, &mut file)
        .unwrap();
}