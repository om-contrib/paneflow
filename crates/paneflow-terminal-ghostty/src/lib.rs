#![deny(unsafe_op_in_unsafe_fn)]

//! `paneflow_ghostty_native` (emitted by `build.rs`) means: the `native`
//! feature is on **and** this target has a reviewed libghostty-vt archive, so
//! the real engine is compiled instead of `stub`. Never re-derive that
//! disjunction at a use site - adding a platform must stay a one-place change
//! (macOS libghostty EP-001 US-001).

// The cfg is computed in build.rs from the same target inputs that decide
// whether the `-sys` crate is a dependency at all (the target-specific table
// in Cargo.toml). Those two must agree. If the cfg is set while the crate is
// absent the failure is loud - unresolved imports - but the opposite drift is
// silent: the stub would be compiled while the archive is linked in, and the
// terminal would quietly do nothing. Pin the invariant here.
#[cfg(all(paneflow_ghostty_native, not(feature = "native")))]
compile_error!(
    "`paneflow_ghostty_native` is set without the `native` feature: \
     crates/paneflow-terminal-ghostty/build.rs and Cargo.toml have diverged"
);

#[cfg(paneflow_ghostty_native)]
mod encode;
#[cfg(all(test, paneflow_ghostty_native))]
mod encode_tests;
mod error;
mod input;
#[cfg(paneflow_ghostty_native)]
mod input_map;
#[cfg(paneflow_ghostty_native)]
mod limits;
mod model;
#[cfg(paneflow_ghostty_native)]
mod osc52;
#[cfg(paneflow_ghostty_native)]
mod osc7;

#[cfg(paneflow_ghostty_native)]
mod abi;
#[cfg(paneflow_ghostty_native)]
mod abi_layout;
#[cfg(paneflow_ghostty_native)]
mod callback_ffi;
#[cfg(paneflow_ghostty_native)]
mod callbacks;
#[cfg(paneflow_ghostty_native)]
mod color_query;
#[cfg(paneflow_ghostty_native)]
mod constructor;
#[cfg(paneflow_ghostty_native)]
mod engine;
#[cfg(paneflow_ghostty_native)]
mod grid;
#[cfg(paneflow_ghostty_native)]
mod handles;
#[cfg(paneflow_ghostty_native)]
mod navigation;
#[cfg(paneflow_ghostty_native)]
mod persistence;
#[cfg(paneflow_ghostty_native)]
mod search;
#[cfg(paneflow_ghostty_native)]
mod snapshot;
#[cfg(paneflow_ghostty_native)]
mod snapshot_cell;
#[cfg(paneflow_ghostty_native)]
mod snapshot_ffi;
#[cfg(paneflow_ghostty_native)]
mod snapshot_state;
#[cfg(not(paneflow_ghostty_native))]
mod stub;

pub use error::{GhosttyError, Result};
pub use input::{
    FocusEvent, Key, KeyAction, KeyInput, Modifiers, MouseAction, MouseButton, MouseInput,
};
pub use model::{
    BackendEvent, Cell, CellFlags, Color, Content, Cursor, CursorShape, Hyperlink, Modes, Point,
    Rgb, Scroll, SearchMatch, SearchResult, SelectionRange, UnderlineStyle, WideCell, WindowSize,
};
#[cfg(paneflow_ghostty_native)]
pub const GHOSTTY_APP_VERSION: &str = paneflow_libghostty_sys::GHOSTTY_APP_VERSION;

#[cfg(paneflow_ghostty_native)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct BuildIdentity {
    pub source_sha: &'static str,
    pub api_version: &'static str,
    pub zig_version: &'static str,
    pub optimization: &'static str,
    pub simd: &'static str,
}

#[cfg(paneflow_ghostty_native)]
pub fn build_identity() -> BuildIdentity {
    const MANIFEST: &str = include_str!("../../../native/libghostty/manifest.toml");

    fn value(key: &str) -> Option<&'static str> {
        let prefix = format!("{key} = \"");
        MANIFEST
            .lines()
            .find_map(|line| line.strip_prefix(&prefix)?.strip_suffix('"'))
    }

    BuildIdentity {
        source_sha: value("source_sha").unwrap_or("unknown"),
        api_version: paneflow_libghostty_sys::EXPECTED_API_VERSION,
        zig_version: value("zig_version").unwrap_or("unknown"),
        optimization: value("build_mode").unwrap_or("unknown"),
        simd: value("simd_profile").unwrap_or("unknown"),
    }
}

#[cfg(paneflow_ghostty_native)]
pub use engine::DisplayTerminal;
#[cfg(not(paneflow_ghostty_native))]
pub use stub::DisplayTerminal;

#[cfg(all(test, paneflow_ghostty_native))]
mod identity_tests {
    #[test]
    fn build_identity_is_derived_from_the_pinned_manifest() {
        let identity = super::build_identity();
        assert_eq!(identity.source_sha.len(), 40);
        assert_eq!(identity.api_version, "0.1.0");
        assert_eq!(identity.zig_version, "0.15.2");
        assert_eq!(identity.optimization, "ReleaseFast");
        assert_eq!(identity.simd, "upstream-default");
    }
}
