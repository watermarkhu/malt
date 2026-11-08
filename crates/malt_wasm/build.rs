//! Compile the freestanding C shim (`shim.c`) so the tree-sitter C parser's
//! libc needs resolve to wasm-bindgen's allocator and no-op stubs instead of
//! the WASI libc (which would emit browser-incompatible `env` imports).
//!
//! Only compiles when building for wasm32; on host builds the shim is skipped.

fn main() {
    let target_arch = std::env::var("CARGO_CFG_TARGET_ARCH").unwrap_or_default();
    if target_arch != "wasm32" {
        return;
    }

    let mut build = cc::Build::new();
    build.file("shim.c");
    build.warnings(false);
    // `__wbindgen_malloc`/`__wbindgen_free` are resolved at link time from the
    // wasm-bindgen exports; compile freestanding so we don't need any sysroot.
    build.compile("malt_shim");
    println!("cargo:rerun-if-changed=shim.c");
}
