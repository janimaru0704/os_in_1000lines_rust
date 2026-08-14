use std::{env, path::PathBuf};

fn main() {
    println!("cargo:rustc-link-arg=-Tkernel.ld");
    println!("cargo:rerun-if-changed=kernel.ld");

    let target_dir = env::var("CARGO_TARGET_DIR").unwrap_or_else(|_| "target".to_string());

    let user_obj = PathBuf::from(target_dir)
        .join("riscv32i-unknown-none-elf")
        .join("debug")
        .join("user.bin.o");

    println!("cargo:rustc-link-arg={}", user_obj.display());
}
