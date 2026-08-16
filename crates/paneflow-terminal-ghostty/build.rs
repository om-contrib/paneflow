//! Build script for `paneflow-terminal-ghostty`.
//!
//! Emits `paneflow_ghostty_native`: *the `native` feature is on and this
//! target has a reviewed libghostty-vt archive*, i.e. the `-sys` crate is
//! linked and the real engine is available rather than the stub.
//!
//! Same reasoning as `paneflow_ghostty` in `src-app/build.rs` (macOS
//! libghostty EP-001 US-001): the predicate was written out at 31 sites in
//! this crate, so adding a platform meant editing every one of them. It is
//! computed here instead, and must stay in sync with the target-specific
//! `[target.'cfg(...)'.dependencies]` entry in Cargo.toml - that entry decides
//! whether the `-sys` crate exists at all, this cfg decides whether the code
//! using it is compiled.

fn main() {
    // Required so `unexpected_cfgs` accepts the custom cfg without a blanket
    // `#[allow]` at the crate root.
    println!("cargo:rustc-check-cfg=cfg(paneflow_ghostty_native)");

    if std::env::var_os("CARGO_FEATURE_NATIVE").is_none() {
        return;
    }

    let target_os = std::env::var("CARGO_CFG_TARGET_OS").unwrap_or_default();
    let target_arch = std::env::var("CARGO_CFG_TARGET_ARCH").unwrap_or_default();

    // macOS is Apple Silicon only: releases ship no Intel artifact, and
    // narrowing here is what keeps an Intel Mac building Alacritty-only
    // instead of failing to link a `-sys` crate it never gets.
    let supported = matches!(target_os.as_str(), "linux" | "windows")
        || (target_os == "macos" && target_arch == "aarch64");

    if supported {
        println!("cargo:rustc-cfg=paneflow_ghostty_native");
    }
}
