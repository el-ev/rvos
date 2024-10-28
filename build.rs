fn main() {
    println!("cargo:rustc-link-search=native=target/riscv64gc-unknown-none-elf/debug/");
    println!("cargo:rustc-link-lib=static=fs");
}