use std::env;
use std::path::PathBuf;
use std::process::Command;

fn main() {
    let out_dir = PathBuf::from(env::var("OUT_DIR").unwrap());
    let obj_path = out_dir.join("boot32.o");

    let status = Command::new("nasm")
        .args([
            "-f",
            "elf64",
            "boot32.asm",
            "-o",
            obj_path.to_str().unwrap(),
        ])
        .status()
        .expect("failed to run nasm -- is it installed?");
    assert!(status.success(), "nasm failed to assemble boot32.asm");

    println!("cargo:rustc-link-arg={}", obj_path.display());
    println!("cargo:rerun-if-changed=boot32.asm");
}
