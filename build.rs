//! This build script copies the `memory.x` file from the crate root into
//! a directory where the linker can always find it at build time.
//! For many projects this is optional, as the linker always searches the
//! project root directory -- wherever `Cargo.toml` is. However, if you
//! are using a workspace or have a more complicated build setup, this
//! build script becomes required. Additionally, by requesting that
//! Cargo re-run the build script whenever `memory.x` is changed,
//! updating `memory.x` ensures a rebuild of the application with the
//! new memory settings.

use std::env;
use std::fs::File;
use std::io::Write;
use std::path::PathBuf;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Chip {
    RP2040,
    RP235x,
}

#[cfg(feature = "rp2040")]
pub const TARGET_CHIP: Chip = Chip::RP2040;

#[cfg(not(feature = "rp2040"))]
pub const TARGET_CHIP: Chip = Chip::RP235x;

fn main() {
    // Put `memory.x` in our output directory and ensure it's
    // on the linker search path.
    // load the environment variables from the .env file
    let target = TARGET_CHIP;

    //let file: &str = if TARGET_CHIP == Chip::RP2040 {
    //    "memory-rp2040.x"
    //} else {
    //    "memory-rp235x.x"
    //};
    let file: &str = "memory.x";
    let data: &[u8] = match target {
        Chip::RP2040 => include_bytes!("memory-rp2040.x"),
        Chip::RP235x => include_bytes!("memory-rp235x.x"),
    };
    let out = &PathBuf::from(env::var_os("OUT_DIR").unwrap());
    File::create(out.join(file))
        .unwrap()
        .write_all(data)
        .unwrap();
    println!("cargo:rustc-link-search={}", out.display());

    // By default, Cargo will re-run a build script whenever
    // any file in the project changes. By specifying `memory.x`
    // here, we ensure the build script is only re-run when
    // `memory.x` is changed.
    println!("cargo:rerun-if-changed={}", file);

    println!("cargo:rustc-link-arg-bins=--nmagic");
    println!("cargo:rustc-link-arg-bins=-Tlink.x");
    if target == Chip::RP2040 {
        println!("cargo:rustc-link-arg-bins=-Tlink-rp.x");
    }
    println!("cargo:rustc-link-arg-bins=-Tdefmt.x");
}
