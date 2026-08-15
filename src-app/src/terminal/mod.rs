//! Terminal state and view - PTY management and GPUI view wrapper.
//!
//! Manages the alacritty_terminal `Term`, its `tty` + `EventLoop` (EP-002), and
//! periodic sync. The TerminalView creates a TerminalElement for cell-by-cell
//! rendering.
//!
//! ## `paneflow_ghostty`
//!
//! Ghostty-backed code in this module tree is gated on `paneflow_ghostty`, a
//! cfg emitted by `src-app/build.rs` (`emit_ghostty_backend_cfg`). It means
//! "the native libghostty-vt backend is linked into this build" - a reviewed
//! static archive exists for the target *and* the matching Cargo feature is
//! on. Never re-derive that disjunction at a use site: adding a platform must
//! stay a one-place change (macOS libghostty EP-001 US-001).
//!
//! Gates that express a *platform primitive* rather than backend availability
//! stay written out explicitly - see the `target_os = "linux"` gates in
//! `ghostty_session.rs` and `view.rs`, revisited when the POSIX host is
//! widened to Darwin (EP-003 US-005).

// The alias is computed in `build.rs` from the same target/feature inputs that
// pull in the wrapper crate. If the two ever drift apart the backend would
// vanish silently instead of failing the build, so pin the invariant here.
// (The opposite direction - alias unset while the crate is present - is caught
// by the unused-crate-dependencies path and by the gated `mod` declarations
// below failing to resolve.)
//
// cfg-policy-allow: this guard exists precisely to compare the alias against
// the feature set, so naming both features here is the point, not a relapse
// into per-platform gating. See tests/ghostty_cfg_policy.rs.
#[cfg(all(
    paneflow_ghostty,
    not(any(feature = "libghostty-linux", feature = "libghostty-windows"))
))]
compile_error!(
    "`paneflow_ghostty` is set but no `libghostty-*` feature is enabled: \
     src-app/build.rs and src-app/Cargo.toml features have diverged"
);

#[cfg(test)]
pub(crate) mod backend_corpus;
pub mod blink;
pub mod element;
#[cfg(paneflow_ghostty)]
mod ghostty_session;
#[cfg(all(test, paneflow_ghostty))]
mod ghostty_stress;
mod input;
mod listener;
mod marks;
#[cfg(all(test, target_os = "linux"))]
mod portable_pty_probe;
mod pty_session;
mod search;
mod service_detector;
pub mod shell;
pub mod types;
pub mod view;

pub use listener::{SpikeTermSize, ZedListener};
pub(crate) use pty_session::TerminalSessionBackend;
pub use pty_session::TerminalState;
#[cfg(test)]
pub(crate) use pty_session::{
    start_render_content_timing_probe, take_render_content_lock_durations,
};
pub use service_detector::ServiceInfo;
pub use view::{TerminalEvent, TerminalView};

#[cfg(debug_assertions)]
pub(crate) use view::probe_enabled;
