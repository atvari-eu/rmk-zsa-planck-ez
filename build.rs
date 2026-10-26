//! Build script: compresses `vial.json` into the Vial keyboard def blob and
//! sets the linker script arguments.

use std::fs::File;
use std::io::Read;
use std::path::Path;
use std::{env, fs};

use const_gen::*;
use xz2::read::XzEncoder;

fn main() {
    // Link our own memory.x: it reserves the last 4 KB of flash for RMK
    // storage. embassy-stm32's `memory-x` feature must stay disabled, or it
    // would generate a full-size memory.x instead.
    let out_dir = env::var_os("OUT_DIR").unwrap();
    let out = Path::new(&out_dir);
    fs::copy("memory.x", out.join("memory.x")).unwrap();
    println!("cargo:rustc-link-search={}", out.display());
    println!("cargo:rerun-if-changed=memory.x");

    // Generate vial config at the root of project
    println!("cargo:rerun-if-changed=vial.json");
    generate_vial_config();

    // `--nmagic` is required if memory section addresses are not aligned to 0x10000,
    // for example the FLASH and RAM sections in your `memory.x`.
    // See https://github.com/rust-embedded/cortex-m-quickstart/pull/95
    println!("cargo:rustc-link-arg=--nmagic");

    // Set the linker script to the one provided by cortex-m-rt.
    println!("cargo:rustc-link-arg=-Tlink.x");

    // Set the extra linker script from defmt
    println!("cargo:rustc-link-arg=-Tdefmt.x");
}

fn generate_vial_config() {
    // Generated vial config file
    let out_file = Path::new(&env::var_os("OUT_DIR").unwrap()).join("config_generated.rs");

    let p = Path::new("vial.json");
    let mut content = String::new();
    match File::open(p) {
        Ok(mut file) => {
            file.read_to_string(&mut content).expect("Cannot read vial.json");
        }
        Err(e) => println!("Cannot find vial.json {:?}: {}", p, e),
    };

    let vial_cfg = json::stringify(json::parse(&content).unwrap());
    let mut keyboard_def_compressed: Vec<u8> = Vec::new();
    XzEncoder::new(vial_cfg.as_bytes(), 6)
        .read_to_end(&mut keyboard_def_compressed)
        .unwrap();

    // Fresh Vial keyboard ID for this port (random 8 bytes).
    let keyboard_id: Vec<u8> = vec![0x50, 0x1A, 0xC3, 0xEE, 0x07, 0xD2, 0x9A, 0x41];
    let const_declarations = [
        const_declaration!(pub VIAL_KEYBOARD_DEF = keyboard_def_compressed),
        const_declaration!(pub VIAL_KEYBOARD_ID = keyboard_id),
    ]
    .map(|s| "#[allow(clippy::redundant_static_lifetimes)]\n".to_owned() + s.as_str())
    .join("\n");
    fs::write(out_file, const_declarations).unwrap();
}
