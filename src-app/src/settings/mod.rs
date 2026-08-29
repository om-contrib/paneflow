//! Embedded settings - Codex-style settings rendered *inside* the main window
//! (grouped nav rail + content panel) rather than a separate GPUI window.
//!
//! Layout:
//! - `chrome`     - the nav rail (`render_settings_nav`) + content panel
//!   (`render_settings_content_panel`) + section dispatch, all on `PaneFlowApp`.
//! - `components` - shared UI primitives (cards, toggles, section headers).
//! - `modal`      - the centered modal card that frames both halves.
//! - `tabs`       - per-section bodies (`general`, `appearance`, `shortcuts`,
//!   `terminal`, `ai_agent`, `mcp`), each `impl PaneFlowApp`.
//!
//! The Settings button (`PaneFlowApp::open_settings_window`, in `app::settings`)
//! sets `settings_section = Some(General)`; `main.rs` then layers
//! `render_settings_modal` over the workspace, which is left untouched
//! underneath. There is no standalone settings window anymore.

pub mod chrome;
pub mod components;
pub mod modal;
pub mod tabs;
