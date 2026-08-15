# Repository Guidelines

## Project Structure & Module Organization
PaneFlow is a Rust workspace with 13 members. `src-app/` contains the `paneflow` desktop binary and CLI entrypoint: UI, terminal rendering, pane management, IPC, themes, and bundled helper binaries under `src-app/assets/`. `crates/paneflow-*` contains the shared config, IPC-client, process, telemetry, ACP, shim, AI-hook, MCP, and MCP-installer crates, plus the three terminal-engine crates (`paneflow-libghostty-sys`, `paneflow-terminal-ghostty`, `paneflow-ghostty-smoke`). `native/libghostty/` holds the pinned Ghostty manifest and the reviewed prebuilt static archives. Top-level `assets/` holds desktop packaging assets, `packaging/` the repo layouts, `schemas/` the public JSON schema, `mcps/paneflow/tools/` the MCP tool manifests, `fuzz/` the cargo-fuzz targets, `scripts/` utility scripts, `docs/` user and release documentation, and `tasks/` PRDs and story status files.

`default-members = ["src-app"]`, so bare `cargo run` / `cargo build` target the desktop app rather than becoming ambiguous across the helper binaries.

## Terminal backends
Terminal emulation is dual-engine. `src-app/Cargo.toml` sets `default = ["libghostty-linux", "libghostty-windows"]`, so `terminal.backend = "auto"` resolves to the statically linked `libghostty-vt` engine on Linux and Windows x64 MSVC. Upstream `alacritty_terminal` is the macOS backend and the explicit cross-platform rollback (`terminal.backend = "alacritty"`). The choice applies to new sessions only - a live session never switches engine, and a Ghostty startup failure may fall back only before the shell child is spawned. `cargo build` must never fetch, compile, or mutate Ghostty artifacts: it only verifies and links the reviewed archives.

## Build, Test, and Development Commands
Run all commands from the repository root.

- `cargo build` builds the workspace.
- `cargo build --release` builds the optimized app binary.
- `cargo run -p paneflow-app` launches the app locally.
- `RUST_LOG=info cargo run -p paneflow-app` runs with structured logging enabled.
- `cargo test --workspace` runs unit and integration tests across every crate.
- `cargo test -p paneflow-app --test flex_nchild -- --nocapture` runs the GPUI layout integration tests only.
- `cargo clippy --workspace -- -D warnings` treats lint warnings as errors.
- `cargo fmt --check` verifies formatting.
- `RUST_LOG=paneflow::terminal::backend=info cargo run` prints the resolved terminal backend.

GPUI and the other Zed crates are **git dependencies** pinned to an exact revision of the Paneflow Zed fork (`arthjean/zed`); Cargo fetches them automatically and no local checkout is required. `alacritty_terminal` is plain upstream crates.io, not a fork. Do not convert either to a local path dependency or to a different crates.io source.

## Coding Style & Naming Conventions
Use standard Rust formatting with `cargo fmt`; the codebase follows 4-space indentation and Rust defaults. Keep modules and files in `snake_case` (`terminal_element.rs`, `config_writer.rs`), types in `UpperCamelCase`, and functions/tests in `snake_case`. Prefer small, focused modules and brief doc comments where behavior is not obvious. Inline GPUI styling is the established pattern; match existing builder-chain style instead of introducing a separate styling layer.

## Testing Guidelines
Add unit tests alongside the module when logic is self-contained, as in `src-app/src/workspace.rs` and `crates/paneflow-config/src/*.rs`. Keep broader UI/layout checks in `src-app/tests/`. Name tests descriptively, for example `test_three_children_flex_basis`. Run `cargo test --workspace`, `cargo clippy`, and `cargo fmt --check` before opening a PR. UI changes should still include manual verification because visual smoke CI is useful but not exhaustive.

## Pre-commit checks (mandatory)

**Before EVERY `git commit` and EVERY `git push` that touches Rust code, run `cargo fmt --check`.** If it reports a diff, run `cargo fmt`, re-stage, then commit.

This is the cheapest guard against the most expensive CI failure on this repo: the release pipeline runs `cargo fmt --check` on all four Build jobs (Linux x86_64, Linux aarch64, macOS aarch64, Windows x86_64) - a single mis-formatted line fails all four legs, skips "Publish GitHub Release", and burns a ~25 min run for nothing. Tag-push releases are extra-painful: a dirty tag commit forces a tag delete + re-create at the fix commit because the original tagged build can't be salvaged. Run `cargo fmt --check` one last time on the exact commit you're about to tag, before `git tag` and `git push origin <tag>`.

## Commit & Pull Request Guidelines
Recent history uses Conventional Commit-style prefixes plus scope, for example `feat(app): US-004 - adapt paneflow-hook for Codex PID env var` and `chore(tasks): ...`. Follow `type(scope): description`; include the story ID when work maps to a tracked task. PRs should explain user-visible behavior, list validation steps, link the relevant issue or PRD entry, and include screenshots or short recordings for UI changes.

## Configuration Notes
Do not replace the pinned Zed git dependencies with crates.io versions or local paths. Linux, macOS Apple Silicon, and Windows x64 all ship as release artifacts - treat all three as active targets. Config resolves through `dirs::config_dir()`: `~/.config/paneflow/paneflow.json` on Linux, `~/Library/Application Support/paneflow/paneflow.json` on macOS, and `%APPDATA%\paneflow\paneflow.json` on Windows. Every key is optional and the public reference lives in `docs/user/configuration/schema.md` and `schemas/`.

Files under `docs/user/` are a **generated mirror** of paneflow.dev, regenerated by `scripts/sync-public-docs.ts` in the site repo. Do not hand-edit them; change the site source and re-sync.

## Cross-platform compatibility (mandatory)

Any new code, refactor, or change that touches the codebase in any way **must** be fully compatible with all three target platforms:

- **Linux** - every major distribution (Fedora, Ubuntu/Debian, Arch, openSUSE, etc.), both Wayland and X11.
- **macOS (Apple)** - Intel and Apple Silicon.
- **Windows** - Windows 10 and 11 (x64, and ARM64 where applicable).

Always verify every implementation decision against Windows, macOS, and Linux compatibility before considering the work done. For Linux, check the behavior against the major distro families and desktop stacks the project targets: Fedora, Ubuntu/Debian, Arch, openSUSE, Wayland, and X11.

Concretely this means:

- Never hardcode POSIX-only paths, shell commands, env vars, or separators. Use `std::path::PathBuf`, `std::env`, and the `dirs` crate (or equivalent) for all filesystem and environment access.
- Guard platform-specific code with `#[cfg(target_os = "…")]` and always provide a working path for the other two platforms (at minimum a graceful fallback or documented stub).
- Prefer cross-platform crates (`portable-pty`, `notify`, `dirs`, `which`, etc.) over POSIX-only APIs. If a POSIX-only crate is unavoidable, isolate it behind a trait with per-OS implementations.
- PTY, IPC, packaging, auto-update, keybindings, fonts, and file watching must each have Linux + macOS + Windows paths - never Linux-only.
- Before shipping a change, mentally (or actually) verify it compiles and behaves correctly on all three platforms. If you cannot verify, say so explicitly rather than assume.

The project is actively porting to macOS and Windows, so all new work must land cross-platform by default.

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
