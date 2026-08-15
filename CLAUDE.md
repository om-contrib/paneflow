# CLAUDE.md - PaneFlow

Native Rust terminal workspace for running coding agents in parallel. Built with Zed's GPUI framework. Terminal emulation runs on a **dual-engine** stack: a pinned, statically linked `libghostty-vt` backend by default on Linux and Windows x64 MSVC, with upstream `alacritty_terminal` (crates.io) as the macOS backend and the explicit cross-platform rollback. Linux, macOS Apple Silicon, and Windows x64 all ship as release artifacts today.

## Commands

```bash
# Build
cargo build
cargo build --release          # LTO thin, strip, codegen-units=1

# Run
cargo run                      # debug build, needs GPUI GPU support (Vulkan)
RUST_LOG=info cargo run        # with logging (env_logger)
RUST_LOG=paneflow::terminal::backend=info cargo run  # log resolved terminal backend
PANEFLOW_LATENCY_PROBE=1 cargo run  # keystroke→pixel latency tracing (debug only)

# Test
cargo test --workspace         # all workspace tests
cargo test -p paneflow-config  # config crate tests only
cargo test -p paneflow-app --test flex_nchild -- --nocapture  # GPUI layout integration tests
cargo test <test_name> -- --nocapture  # single test with output

# Lint
cargo clippy --workspace -- -D warnings
cargo fmt --check
```

`default-members = ["src-app"]`, so bare `cargo run` / `cargo build` target the desktop app instead of becoming ambiguous across the helper binaries.

### Fork-pin maintenance (Zed Markdown widget)

The eight Zed git deps in `src-app/Cargo.toml` pin `arthjean/zed@3aaba57b95c22f4d21bbbf9f4b10b513173209db`, published from `paneflow/gpui-2026-07-14`. That commit is based on `zed-industries/zed@afc13dc8` and carries the Paneflow Markdown streaming optimization. To bump it, choose and freeze a tested upstream revision, create a new dated fork branch from that revision, reapply the Markdown patch while preserving Zed's current public API, validate and publish the fork commit, update all eight exact `rev` values, run `cargo update`, then run the workspace test, Clippy, and format gates. Once the optimization lands upstream, switch all eight entries back to `zed-industries/zed` at the exact tested merge revision.

### Perf / heap profiling

`tasks/heaptrack-runbook.md` documents the heaptrack procedure, but it explicitly labels itself a **placeholder for re-running the measurement, not a validated current benchmark** - the agent UI paths its source draft cited have since moved. Re-measure before quoting any RAM number publicly, and record the exact commit, OS, GPU backend, and scenario alongside it. CPU work uses `cargo flamegraph`; the keystroke-latency probe (`PANEFLOW_LATENCY_PROBE=1`, debug builds) covers input→pixel.

## Pre-commit checks (mandatory)

**Before EVERY `git commit` and EVERY `git push` that touches Rust code, run:**

```bash
cargo fmt --check
```

If it reports any diff, run `cargo fmt`, re-stage the touched files, then commit.

Why this is non-negotiable on this repo:

- The release pipeline (`.github/workflows/release.yml`) runs `cargo fmt --check` as step 9 of every Build job on all four matrix legs (Linux x86_64, Linux aarch64, macOS aarch64, Windows x86_64). A single mis-formatted line fails all four legs, skips the "Publish GitHub Release" step, and burns a ~25 min CI run before producing nothing.
- It also blocks tag-push releases: if the tag commit is dirty, you have to delete + re-create the tag at the fix commit (force-update) to retry - the original tagged build cannot be salvaged.
- rustfmt drifts between Rust point releases (`a4c75f6` was a v0.2.15 patch for rustfmt 1.9.0; `c292dfa` was the same patch for v0.2.16). Even code that compiled clean a week ago can need re-formatting after a toolchain bump.

For tag-push releases specifically: run `cargo fmt --check` *one last time* on the exact commit you're about to tag, before `git tag` and `git push origin <tag>`. This is the cheapest possible guard against a wasted 25 min release run.

## Workspace crates

13 members: one binary crate plus twelve focused libraries. Anything that runs **outside** the GUI process (shim, hook, MCP bridge, MCP installer) stays GPU-free and never links GPUI.

| Crate | Path | Type | Purpose |
|---|---|---|---|
| `paneflow-app` | `src-app/` | Binary | GPUI application + `paneflow` CLI entrypoint: UI, panes, PTY sessions, IPC server, self-update |
| `paneflow-libghostty-sys` | `crates/paneflow-libghostty-sys/` | Library | Raw Ghostty ABI, verification + linking of the pinned static archive |
| `paneflow-terminal-ghostty` | `crates/paneflow-terminal-ghostty/` | Library | Safe Rust interface over Ghostty state, input, search, selection, owned render snapshots |
| `paneflow-ghostty-smoke` | `crates/paneflow-ghostty-smoke/` | Binary | Package-level native smoke: Ghostty + PTY I/O + resize + shutdown |
| `paneflow-config` | `crates/paneflow-config/` | Library | Config schema, tolerant JSON loader, file watcher |
| `paneflow-shim` | `crates/paneflow-shim/` | Binary | PATH shim wrapping 16 known agent CLIs to observe their lifecycle |
| `paneflow-ai-hook` | `crates/paneflow-ai-hook/` | Binary | Hook binary agent CLIs invoke to report session events over IPC |
| `paneflow-ipc-client` | `crates/paneflow-ipc-client/` | Library | Blocking JSON-RPC client for the local socket (shared by MCP bridge + CLI) |
| `paneflow-mcp` | `crates/paneflow-mcp/` | Binary | Stdio MCP server: read-only pane access |
| `paneflow-mcp-install` | `crates/paneflow-mcp-install/` | Library | GPU-free install engine: per-agent detection, idempotent merge, backup + atomic write |
| `paneflow-process` | `crates/paneflow-process/` | Library | Bounded external-process execution (wall-clock deadline + stdout cap) |
| `paneflow-acp` | `crates/paneflow-acp/` | Library | Legacy Claude/Codex identity enum + `CLAUDECODE` env scrub |
| `paneflow-telemetry` | `crates/paneflow-telemetry/` | Library | Opt-in telemetry plumbing (nothing leaves the machine without explicit consent) |

Non-crate directories that matter: `native/libghostty/` (pinned Ghostty source manifest + reviewed prebuilt static archives per target), `fuzz/` (cargo-fuzz targets), `mcps/paneflow/tools/` (MCP tool manifests), `schemas/` (public JSON schema), `packaging/`, `skills/`, `docs/`, `tasks/`.

## Architecture

```
PaneFlowApp (Entity<Render>)           ← src-app/src/main.rs
├── app/                               ← PaneFlowApp impl, split across modules
│   ├── actions.rs                     ← ~85 GPUI action types (paneflow namespace)
│   ├── bootstrap.rs                   ← app init, window creation, GPUI setup
│   ├── event_handlers.rs              ← title-bar/pane/terminal subscribers + stale-PID sweep
│   ├── ipc_handler.rs                 ← JSON-RPC handler dispatched to GPUI main thread
│   ├── session.rs                     ← persist/restore workspaces to session.json
│   ├── settings.rs                    ← settings lifecycle: open/close, persist_setting, keys
│   ├── self_update_flow.rs            ← check/download/install orchestration
│   ├── attention_queue.rs             ← prioritized "who needs you" queue
│   ├── broadcast.rs / composer.rs     ← multi-pane send, prompt composer
│   ├── agents_view_actions.rs         ← Agents-mode actions (largest app module)
│   ├── agents_bottom_panel.rs         ← docked agent terminals
│   ├── diff_view_actions.rs / diff_view_helpers.rs
│   ├── files_tree.rs / fleet_search.rs / launch_pad.rs
│   ├── notifications.rs / theme_picker.rs / profile_menu.rs
│   ├── custom_buttons_modal.rs / sidebar_actions_menu.rs / about_dialog.rs
│   ├── constants.rs / drag.rs / telemetry_events.rs
│   ├── sessions_sidebar.rs            ← agent session discovery/resume dock
│   ├── agents_diff/ agents_sidebar/ diff_sidebar/ files_sidebar/
│   └── sidebar/ project_ops/ workspace_ops/
├── window_chrome/
│   ├── csd.rs                         ← client-side decorations, resize edges
│   ├── title_bar.rs                   ← window controls, drag-to-move
│   └── backdrop.rs / linux_backdrop.rs / macos_backdrop.rs  ← per-OS material
├── workspace/                         ← Vec<Workspace> state
│   ├── mod.rs                         ← Workspace struct, AI agent PIDs, ports
│   ├── git.rs / worktree.rs           ← branch detection, worktree scans
│   ├── ports.rs                       ← TCP port scan (Linux /proc, macOS libproc, Windows)
│   ├── pid_resolve.rs / surface_naming.rs
├── layout/                            ← N-ary tree of panes
│   ├── tree.rs / mutations.rs / navigation.rs / close.rs
│   ├── presets.rs                     ← even_h, even_v, main_vertical, tiled
│   ├── render.rs                      ← GPUI flex emission
│   └── queries.rs / serde.rs
├── pane.rs / pane_drag.rs             ← Pane: tab strip + active terminal, tab drag
├── terminal/                          ← PTY session + VT emulation + rendering
│   ├── view.rs                        ← TerminalView (Entity<Render>)
│   ├── ghostty_session.rs             ← libghostty-vt session (default Linux/Windows x64)
│   ├── pty_session.rs                 ← Alacritty session (macOS + rollback)
│   ├── backend_corpus.rs              ← Ghostty/Alacritty differential corpus
│   ├── ghostty_stress.rs / portable_pty_probe.rs
│   ├── listener.rs / input.rs         ← ZedListener, keystroke translation
│   ├── search.rs / marks.rs           ← find-in-buffer, OSC 133 prompt marks
│   ├── service_detector.rs / shell.rs ← dev-server detection, shell resolution
│   ├── blink.rs / types.rs            ← cursor blink, shared terminal types
│   └── element/                       ← low-level GPUI Element rendering
│       ├── mod.rs                     ← TerminalElement: layout → prepaint → paint
│       ├── color.rs                   ← ANSI→Hsla, APCA contrast
│       ├── font.rs / geometry.rs      ← font resolution, cell geometry
│       ├── hyperlink.rs               ← OSC 8 + URL scanning
│       ├── pixel_probe.rs             ← latency probe hook
│       └── paint/ golden/             ← paint-pass helpers, golden snapshots
├── diff/                              ← Review + Agents diff engine
│   ├── engine.rs / rows.rs / element.rs   ← imara-diff pipeline, row model, direct paint
│   ├── git.rs / extract.rs / scope.rs     ← worktree scans, blob extraction, scoping
│   ├── syntax.rs / highlighter.rs         ← Tree-sitter highlighting
│   ├── hscroll.rs / hit_test.rs / align.rs / arrange.rs / worddiff.rs
│   └── view.rs / multi_view.rs / view/    ← Review column + multi-worktree columns
├── markdown/                          ← markdown panes (Zed markdown widget wrapper)
│   ├── parser.rs / state.rs / view.rs / theme.rs / security.rs
├── cli/                               ← `paneflow` CLI subcommands
│   ├── up_cmd.rs / flow_cmd.rs / flow_spec.rs / workspace_spec.rs
│   ├── send_cmd.rs / wait_cmd.rs / watch_cmd.rs / read_cmds.rs
│   └── control_cmds.rs / selector.rs
├── agents/                            ← agent notifications, parent-process guard
├── agents_view/                       ← Agents mode + skills
├── ai_hooks/                          ← hook payload extraction
├── project/                           ← project detection + git stats
├── theme/                             ← theme model + hot-reload (5 bundled themes)
│   ├── model.rs                       ← TerminalTheme + UiColors + ui_colors()
│   ├── builtin.rs                     ← 5 themes + THEMES table + theme_by_name
│   └── watcher.rs                     ← 500 ms mtime cache, active_theme()
├── keybindings/
│   ├── defaults.rs / registry.rs      ← default bindings, action registry
│   ├── apply.rs                       ← apply_keybindings() wires cx.bind_keys
│   └── display.rs                     ← human-readable binding strings
├── settings/                          ← embedded settings (inline, not a window)
│   ├── chrome.rs / nav_header.rs      ← grouped nav rail + content panel
│   ├── components.rs                  ← shared cards/toggles/section headers
│   └── tabs/                          ← general, appearance, shortcuts, terminal,
│                                        ai_agent, mcp, notifications, workspaces
├── update/                            ← self-update
│   ├── checker.rs / error.rs          ← release checker, structured UpdateError
│   ├── install_method.rs              ← detect install mode (AppImage/.deb/.msi/.app/.tar.gz)
│   ├── signature.rs / verified_download.rs  ← minisign verify, fail-closed download
│   ├── migrations.rs                  ← legacy install-layout migration
│   └── linux/ / macos/ / windows/     ← per-OS install paths (incl. MSI relay)
├── widgets/                           ← text_input, text_area, scrollbar, callout
├── telemetry/                         ← app-side telemetry id + tags
├── agent_launcher.rs                  ← agent CLI launch + shim wiring
├── agent_sessions.rs                  ← unified agent session model
├── claude_sessions.rs / codex_sessions.rs / opencode_sessions.rs / pi_sessions.rs
├── command_sessions.rs                ← plain shell-command panes
├── ai_types.rs                        ← AiToolState enum shared by workspace/event_handlers
├── ipc.rs / ipc_events.rs             ← JSON-RPC server + event bus (cross-platform)
├── keys.rs / mouse.rs                 ← key/mouse translation
├── search.rs                          ← find-in-buffer UI glue
├── runtime_paths.rs                   ← XDG + %APPDATA% path helpers
├── config_writer.rs                   ← read-modify-write paneflow.json
├── fonts.rs                           ← font loading (bundled Geist Mono + system fallback)
├── editor.rs / external_open.rs       ← external editor + file-manager launch
├── login_shell_env.rs / launch_cwd.rs / limits.rs / pricing.rs
├── ui_primitives.rs / window_state.rs / windows_app_identity.rs
└── assets.rs                          ← rust-embed asset registry
```

### Thread model

- **Main thread**: GPUI event loop - owns all Entity state, rendering, input dispatch. No locks around UI state.
- **Terminal workers**: the backend is chosen **before** the shell child is created. Ghostty sessions own a dedicated runtime worker plus a PTY reader; Windows adds a dedicated ConPTY closer so pipe drainage cannot block teardown. Alacritty sessions keep their `EventLoop` I/O thread and the shared `Arc<FairMutex<Term<ZedListener>>>` grid. Both publish backend-neutral events and **owned** render snapshots through `TerminalSessionBackend`.
- **IPC thread**: socket/named-pipe server. Stateless methods reply in place; stateful methods dispatch to the main thread through a bounded channel drained by the 50 ms app poll loop.
- **Watcher threads**: config, theme, git state.
- Blocking work (git subprocesses, filesystem walks, fleet-wide search) runs on background executors. The render thread never does blocking I/O.

### Data flow: keystroke → pixel

```
KeyDownEvent
  → TerminalView::handle_key_down()
  → Ghostty structured input  |  Alacritty escape-sequence input (keys::to_esc_str)
  → selected backend writer → PTY → shell / agent CLI
  → output bytes → libghostty-vt engine  |  Alacritty VTE + Term grid
  → TerminalBackendEvent → sync() → cx.notify()
  → TerminalSessionBackend::render_content() → owned neutral Content
  → TerminalElement::prepaint()
  → TerminalElement::paint()   - paint_quad + shape_line
  → GPU (Vulkan on Linux, Metal on macOS, DirectX on Windows)
```

The first Ghostty wakeup on Linux renders immediately. Windows Ghostty and all Alacritty wakeups are coalesced into the 4 ms event batch.

## Terminal backends (dual engine)

`TerminalSessionBackend` is the renderer-facing facade for both engines.

- `src-app/Cargo.toml` sets `default = ["libghostty-linux", "libghostty-windows"]`.
- `terminal.backend = "auto"` resolves to **Ghostty** on standard Linux builds and supported Windows x64 MSVC builds; to **Alacritty** on macOS, on builds without a verified native Ghostty feature, and on explicit rollback.
- `terminal.backend = "alacritty"` is the documented rollback; `"ghostty"` forces the engine for diagnosis. **The choice applies to new sessions only** - a live session never switches engine.
- A Ghostty startup failure may fall back to Alacritty **only before the shell child exists**. Once a child is spawned, Paneflow never starts a second child.
- Raw ABI + static archive linking live in `paneflow-libghostty-sys`; the safe Rust surface lives in `paneflow-terminal-ghostty`. Alacritty imports are confined to an explicit allowlist. Neither engine leaks borrowed terminal state into GPUI - the app consumes Paneflow-owned points, mode flags, cells, events, and `Content` snapshots.
- `native/libghostty/manifest.toml` is the single source of truth for the pinned inputs (Ghostty `ae52f97d`, Zig 0.15.2, ReleaseFast). Cargo **never** fetches Ghostty, runs bindgen, or invokes Zig - it only verifies and links reviewed archives under `native/libghostty/prebuilt/<rust-target>/`.
- Parity is enforced by a differential corpus in `terminal/backend_corpus.rs` (exact-parity cases plus explicitly pinned semantic differences) and two `cargo-fuzz` targets.

## Layout system (`layout/`)

- Panes form an **N-ary tree** (`layout/tree.rs`), not a binary split tree. The old `split.rs` / `SplitNode` binary tree is gone.
- `SplitDirection::Horizontal` means a horizontal divider bar (panes stacked top/bottom, `flex_col`); `Vertical` means side-by-side (`flex_row`). Counterintuitive, but consistent throughout.
- Rendering emits GPUI flex divs (`layout/render.rs`). Min pane size 80px, ratio clamped 0.1-0.9, divider is a 4px bar.
- Presets in `layout/presets.rs`: `even_h`, `even_v`, `main_vertical`, `tiled`.
- Budgets live in `limits.rs` (pane and workspace caps) and are re-validated on session restore.
- Focus navigation is structural tree traversal (`layout/navigation.rs`), not spatial.

## Critical external dependencies

GPUI and related crates are **git dependencies** pinned to the Paneflow Zed fork while the markdown streaming patch is in flight (eight entries, all at the same rev):

```toml
gpui          = { git = "https://github.com/arthjean/zed", rev = "3aaba57b95c22f4d21bbbf9f4b10b513173209db" }
gpui_platform = { git = "https://github.com/arthjean/zed", rev = "3aaba57b95c22f4d21bbbf9f4b10b513173209db" }
collections   = { git = "https://github.com/arthjean/zed", rev = "3aaba57b95c22f4d21bbbf9f4b10b513173209db" }
markdown / theme / ui / ...  # same rev
```

**No local checkout is required** - Cargo fetches from git automatically. Two crates-io patches are required by GPUI:
- `async-task` → `smol-rs/async-task` (specific git commit)
- `calloop` → `zed-industries/calloop` fork

`alacritty_terminal = "0.26"` comes from **crates.io upstream** (migrated off the Zed fork). `libghostty-vt` is a reviewed static archive vendored under `native/libghostty/`, not a package dependency.

## GPUI patterns

- **Entity/Context model**: all mutable state lives in `Entity<T>`, mutated via `Context<Self>`. Use `cx.new()` to create, `cx.notify()` to trigger repaint, `cx.spawn()` for async tasks.
- **`actions!` macro**: generates zero-sized typed action structs in the `paneflow` namespace. Actions dispatch through GPUI's focus chain.
- **`Render` trait**: implement for high-level views (PaneFlowApp, TitleBar, TerminalView). Returns a div element tree.
- **`Element` trait**: implemented directly only for low-level custom rendering (`TerminalElement`, `DiffElement`). Three phases: `request_layout()` → `prepaint()` → `paint()`.
- **Focus**: each `TerminalView` owns a `FocusHandle`. Key context `"Terminal"` scopes terminal-only keybindings.
- **No `Arc`/`Mutex` for UI state** - use `Rc<Cell<f32>>` for single-threaded shared state (e.g. split ratios in render closures).

## GPUI scroll & wheel (gotchas)

Hard-won from the diff-dock horizontal-scroll saga (`src-app/src/app/agents_diff/mod.rs`, `src-app/src/diff/hscroll.rs`). Verified against the Zed source. Do NOT re-derive these by guessing - it cost three wrong attempts.

- **Shift+wheel is axis-swapped to X at the platform layer**, before app code ever sees it. X11 (`gpui_linux/.../x11/client.rs::make_scroll_wheel_event`), Wayland (`wayland/client.rs`, forces `HorizontalScroll`), and Windows (`gpui_windows/events.rs`) all put the value in `delta.x` and zero `delta.y` when `modifiers.shift`. So: read `delta.x` for horizontal, NEVER branch on `modifiers.shift` (reading `delta.y` under Shift reads zero). macOS: the NSEvent delivers horizontal natively, same effect. The `div.rs` `delta_x = delta.y` line is a separate fallback (fires only when `delta.x == 0`), not the Shift mechanism.
- **`overflow_hidden()` + `track_scroll()` does NOT scroll-translate children.** It only keeps the handle's bookkeeping (`offset()`/`bounds()`/`max_offset()`) live. GPUI only pushes the scroll offset onto the element-offset stack (which bakes into each child's `bounds.origin`) when the host overflow axis is `Overflow::Scroll`. A custom `Element` that positions content off its own `bounds.origin` (e.g. `DiffElement`) therefore only scrolls under `overflow_y_scroll`/`overflow_scroll`; `set_offset()` under `overflow_hidden` is stored but dead. Custom elements get the shift automatically via their passed `bounds` (no `window.element_offset()` call needed).
- **Two-axis recipe (vertical list whose items also scroll horizontally)** - the canonical Zed pattern (`data_table.rs`, `thread_view.rs`, `markdown.rs`): host = `overflow_y_scroll()` + `track_scroll(&handle)` + `element.style().restrict_scroll_to_axis = Some(true)`. The flag is a raw `StyleRefinement` mutation (no builder method, but it compiles: non-`#[refineable]` `Style` fields still become `Option<T>`). It stops a vertical wheel bleeding into a horizontal child AND stops the native Y handler back-filling `delta_y = delta.x` under Shift+wheel (the "vertical scrolls when I Shift+wheel" bug). Per-item horizontal stays custom (an `on_scroll_wheel` reading `delta.x` only); native owns vertical.

## Keybindings

All registered in `keybindings::apply_keybindings()` via `cx.bind_keys()`. ~85 actions total (see `app/actions.rs`; `keybindings/defaults.rs` is the binding table). Bindings use a `secondary` modifier that GPUI maps to `Cmd` on macOS and `Ctrl` elsewhere.

| Key | Action | Context |
|-----|--------|---------|
| `Ctrl/Cmd+Shift+D/E` | Split horizontal/vertical | Global |
| `Ctrl/Cmd+Shift+W` | Close pane | Global |
| `Alt+Arrow` | Focus navigation | Global |
| `Ctrl/Cmd+Shift+N` | New workspace | Global |
| `Ctrl/Cmd+Shift+Q` | Close workspace | Global |
| `Ctrl+Tab` | Next workspace | Global |
| `Ctrl/Cmd+1-9` | Select workspace | Global |
| `Ctrl/Cmd+Shift+C/V` | Copy/Paste | Terminal |
| `Shift+PageUp/Down` | Scroll | Terminal |

Users override bindings via the `shortcuts` object in `paneflow.json` (keystroke → action name). Full public table: `docs/user/keybindings.md`.

## Config

`dirs::config_dir()`-based, so genuinely cross-platform (`crates/paneflow-config/src/loader.rs:41`):

- Linux: `$XDG_CONFIG_HOME/paneflow/paneflow.json` (`~/.config/paneflow/paneflow.json`)
- macOS: `~/Library/Application Support/paneflow/paneflow.json`
- Windows: `%APPDATA%\paneflow\paneflow.json`

Every key is optional - `{}` is valid. The full public reference is `docs/user/configuration/schema.md` and `schemas/`.

```json
{
  "default_shell": "/bin/zsh",
  "theme": "One Dark",
  "window_decorations": "client",
  "terminal": { "backend": "auto" },
  "shortcuts": {},
  "commands": []
}
```

- **Theme hot-reload**: 500 ms mtime polling in a `cx.spawn` loop. **5 bundled themes**: One Dark (default), PaneFlow Light, Vercel, Claude, Cursor. Presets can define app-wide `UiColors`, not just terminal ANSI slots.
- **`window_decorations`**: read at startup only - requires restart. `"client"` = CSD, `"server"` = SSD.
- **`terminal.backend`**: `auto` (default) | `ghostty` | `alacritty`. New sessions only.
- **`shortcuts`**: applied by `keybindings::apply_keybindings()` at startup.
- **`ConfigWatcher`** (notify crate, 300 ms debounce): background thread detects file changes and deposits new config for the GPUI main thread to apply.
- Session state is separate (`session.json`). Terminal scrollback is **process-local** since 0.8.0 and is no longer serialized.

## IPC (`ipc.rs`)

JSON-RPC 2.0. Unix socket at `<runtime_dir>/paneflow/paneflow.sock`, where `runtime_dir` resolves `$XDG_RUNTIME_DIR` → `dirs::runtime_dir()` → `$TMPDIR` (macOS) → `dirs::cache_dir()/run`. Windows uses the named pipe `\\.\pipe\paneflow`. The composed Unix path is rejected if it would exceed the `sun_path` limit (≤103 bytes usable).

| Namespace | Methods | Thread |
|---|---|---|
| `system.*` | `ping`, `capabilities`, `identify` | Socket (stateless) |
| `workspace.*` | `list`, `current`, `create`, `select`, `close`, `up`, `restore_layout` | GPUI |
| `surface.*` | `list`, `read`, `search`, `focus`, `rename`, `status`, `split`, `send_text`, `send_keystroke` | GPUI |
| `fleet.*` | `list` | GPUI |
| `events.*` | `subscribe` | Event bus |
| `ai.*` | `session_start`, `prompt_submit`, `tool_use`, `notification`, `stop`, `exit`, `session_end` | GPUI (from `paneflow-ai-hook`) |

Stateful methods dispatch to the GPUI main thread via `mpsc::channel`, drained by the app poll loop. Write paths (`send_text`, `send_keystroke`) are gated behind explicit scripting access; `send_keystroke` rejects newline bytes. The `paneflow` CLI (`up`, `flow`, `send`, `wait`, `watch`, `ps`, `read`) is built on the same socket via `paneflow-ipc-client`.

## Styling conventions

- **All styling is inline** via GPUI's Tailwind-like builder API: `.bg(rgb(0x181825)).px_3().rounded_md()`
- **UI chrome colors come from the theme's `UiColors`** since the branded presets landed in 0.7.8 - sidebars, settings, diff surfaces, and syntax follow the active theme. Do not reintroduce hardcoded hex for themable chrome.
- **Terminal colors** use the `TerminalTheme` struct resolved via `active_theme()`.
- **Fonts**: bundled JetBrainsMono Nerd Font Mono is the cross-platform terminal default; `Geist` / `Geist Mono` ship bundled for UI and terminal. `.PaneflowMono` / `.PaneflowSans` resolve to the bundled families. Invalid font names fall back to the first available preferred family.

## Gotchas

- **GPUI is not on crates.io** - it comes from the pinned Zed git fork above. Never replace it with a crates.io dependency, and never convert it to a local path dependency.
- **Never recommend iced** for this project - it was evaluated and rejected (unstable, custom WGPU glyph atlas too complex). The decision is final.
- **`SplitDirection::Horizontal`** means a horizontal divider bar (panes stacked top/bottom), NOT side-by-side.
- **Ghostty is the default engine on Linux and Windows x64 MSVC.** Do not describe Alacritty as "the" terminal engine; it is the macOS backend and the rollback.
- **Never let a raw Ghostty pointer reach GPUI.** The C API's render state can be invalidated the moment the terminal changes - copy into owned Rust snapshots on the worker thread.
- **`dirs` version mismatch**: `src-app` uses `dirs = "5.0"`, `paneflow-config` uses `dirs = "6"`. They coexist but are separate semver releases.
- **Config `default_shell` is wired** - fallback chain: config `default_shell` → `$SHELL` → platform default.
- **Tests + CI exist** - run `cargo test --workspace`, `cargo clippy --workspace -- -D warnings`, and `cargo fmt --check`; UI changes still need manual verification.
- **License** - GPL-3.0-or-later; keep packaging metadata in sync with the root `LICENSE` and `Cargo.toml`.
- **`cargo build` must stay offline w.r.t. Ghostty.** If a change would make the build fetch, compile, or mutate Ghostty artifacts, it is wrong.

## Docs map

- `README.md` - product overview, install, workflows, safety model
- `ARCHITECTURE.md` - runtime architecture, thread model, dual engine, cross-platform matrix
- `AGENTS.md` - repository guidelines for coding agents
- `CHANGELOG.md` - per-release summaries
- `BUILD_WEEK.md` - frozen OpenAI Build Week engineering story (historical, do not "update")
- `docs/mcp-bridge.md`, `docs/WINDOWS.md`, `docs/release-runbook.md`, `docs/release/*`
- `docs/user/**` - **generated mirror** of paneflow.dev, regenerated by `scripts/sync-public-docs.ts` in the site repo. Do not hand-edit; backport to the site source instead.
- `llms.txt` - compact repo map for agents and crawlers

## PRD reference

Active/recent PRDs in `tasks/` (each with a sibling `-status.json` where applicable):

- `prd-linux-libghostty-backend-2026-Q3.md` - Linux Ghostty backend (shipped in 0.8.0)
- `prd-windows-libghostty-backend-2026-Q3.md` - Windows x64 MSVC Ghostty backend (shipped in 0.8.1)
- `prd-linux-libghostty-promotion-2026-Q3.md` - default-backend promotion
- `openai-build-week-final-audit-2026-07-21.md` - submission claim audit
- `diag-agent-hooking-2026-Q3.md` - agent hook diagnostics

`tasks/` also holds launch/marketing drafts (`show-hn-draft.md`, `hn-launch-playbook.md`, `devto-*`, `linkedin-*`) and the demo recording script.

## MCP bridge (`paneflow-mcp`)

`crates/paneflow-mcp/` is a stdio MCP server letting CLI agents (Claude Code, Codex, Gemini, opencode) read other panes' terminal output via the existing IPC socket. Read-only: `list_panes` / `read_pane` / `search_pane`. It defaults to the launching pane's workspace via `PANEFLOW_WORKSPACE_ID`; `PANEFLOW_MCP_SCOPE=all` is the explicit instance-wide opt-in. Returned terminal text is fenced as **untrusted** output. Tool manifests live in `mcps/paneflow/tools/`.

**Distribution (`paneflow mcp install`).** The bridge ships embedded in the `paneflow` binary (staged by `build.rs`, extracted at launch to a stable, non-versioned path under `data_dir()/paneflow/bin/` that survives updates - `runtime_paths::bridge_binary_path()`). `paneflow mcp install | uninstall | status` (intercepted in `main.rs` before GUI init) registers/removes/inspects the `paneflow` MCP entry across every detected agent. The engine is the GPU-free `crates/paneflow-mcp-install/` crate (idempotent, no-clobber, backup + atomic write; `toml_edit` kept out of the embedded bridge per budget). Per-agent shapes: Claude Code `~/.claude.json` `mcpServers` (prefers `claude mcp add`), Codex `~/.codex/config.toml` `[mcp_servers.*]` (prefers `codex mcp add`), Gemini `~/.gemini/settings.json` `mcpServers` (`trust:true`), opencode `~/.config/opencode/opencode.json` key `mcp` (`command` array, `type:local`). Full setup + per-agent config: `docs/mcp-bridge.md`. There is also a Settings → AI Agent → "MCP bridge" button that runs the same install off the render thread (state-aware label: Install / Repair / Reinstall).

## Commit convention

```
feat(module): US-NNN - description
refactor(module): description
docs: description
chore: description
```

Atomic commits per user story. Branch naming: `feat/description`.

## Cross-platform compatibility (mandatory)

Any new code, refactor, or change that touches the codebase in any way **must** be fully compatible with all three target platforms:

- **Linux** - every major distribution (Fedora, Ubuntu/Debian, Arch, openSUSE, etc.), both Wayland and X11.
- **macOS (Apple)** - Apple Silicon is the shipped target; keep Intel paths compiling.
- **Windows** - Windows 10 1809+ and Windows 11 (x64 ships; ARM64 is deferred but must not be broken at compile time).

Concretely this means:

- Never hardcode POSIX-only paths, shell commands, env vars, or separators. Use `std::path::PathBuf`, `std::env`, and the `dirs` crate (or equivalent) for all filesystem and environment access.
- Guard platform-specific code with `#[cfg(target_os = "…")]` and always provide a working path for the other two platforms (at minimum a graceful fallback or documented stub).
- Prefer cross-platform crates (`portable-pty`, `notify`, `dirs`, `which`, etc.) over POSIX-only APIs. If a POSIX-only crate is unavoidable, isolate it behind a trait with per-OS implementations.
- PTY, IPC, packaging, auto-update, keybindings, fonts, and file watching must each have Linux + macOS + Windows paths - never Linux-only.
- Before shipping a change, mentally (or actually) verify it compiles and behaves correctly on all three platforms. If you cannot verify, say so explicitly rather than assume.

## Anti-Friction Rules (claude-doctor)

Règles pour éviter les patterns de friction détectés par `claude-doctor` sur ce projet : edit-thrashing, restart-cluster, repeated-instructions, negative-drift, error-loop, excessive-exploration.

### Editing discipline (anti edit-thrashing)

- Read the full file before editing. Plan all changes, then make ONE complete edit.
- If you've edited the same file 3+ times, STOP. Re-read the user's original requirements and re-plan from scratch.
- Prefer one large coherent edit over multiple small incremental ones.

### Stay aligned with the user (anti repeated-instructions, rapid-corrections)

- Re-read the user's last message before responding. Follow through on every instruction completely - don't partially address requests.
- Every few turns on a long task, re-read the original request to verify you haven't drifted from the goal.
- When the user corrects you: stop, re-read their message, quote back what they actually asked for, and confirm understanding before proceeding.

### Act, don't explore (anti excessive-exploration)

- Don't read more than 3-5 files before making a change. Get a basic understanding, make the change, then iterate.
- Prefer acting early and correcting via feedback over prolonged reading and planning.

### Break loops (anti error-loop, restart-cluster)

- After 2 consecutive tool failures or the same error twice, STOP. Change your approach entirely - don't retry the same strategy. Explain what failed and try something genuinely different.
- When truly stuck, summarize what you've tried and ask the user for guidance rather than retrying.

### Verify output (anti negative-drift)

- Before presenting your result, double-check it actually addresses what the user asked for.
- If the diff doesn't map cleanly to the user's request, don't ship it - re-plan.
