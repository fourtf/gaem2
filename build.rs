extern crate gl_generator;

use gl_generator::{Api, Fallbacks, GlobalGenerator, Profile, Registry};
use std::env;
use std::fs::File;
use std::io::Write;
use std::path::Path;

fn main() {
    native();

    let dest = env::var("OUT_DIR").unwrap();
    let path = Path::new(&dest).join("bindings.rs");

    if !path.exists() {
        let mut file = File::create(&path).unwrap();

        Registry::new(Api::Gl, (3, 3), Profile::Compatibility, Fallbacks::All, [])
            .write_bindings(GlobalGenerator, &mut file)
            .unwrap();
    }
}

#[cfg(windows)]
fn native() {
    println!(r"cargo:rustc-link-search=native=C:\Local\gaemlib");

    let out_dir = env::var("OUT_DIR").unwrap();
    let target = Path::new(&out_dir).join("../../../SDL2.dll");

    if !target.exists() {
        std::fs::copy(&Path::new("C:\\Local\\gaemlib\\SDL2.dll"), &target).unwrap();
    }

    /*
    let source = Path::new(&out_dir).join("../../../../../content").canonicalize().unwrap();
    let target = Path::new(&out_dir).join("../../../contentpath");
    if !target.exists() {
        let mut file = File::create(&target).unwrap();
        write!(file, "{}", source.to_str().unwrap()).unwrap();
    }
    */
}

#[cfg(unix)]
fn native() {
    println!(r"cargo:rustc-link-search=native=/usr/lib/x86_64-linux-gnu");
}
