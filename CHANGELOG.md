# Changelog

Notable changes to Paneflow are summarized here. Release artifacts and full
notes are available on the [GitHub Releases](https://github.com/arthjean/paneflow/releases) page.

## [Unreleased]

## [0.8.2] - 2026-07-21

### Fixed

- The Windows title bar now paints an opaque theme surface whenever its own
  chrome material is disabled, instead of exposing terminal-only Mica when the
  terminal enabled the host-window backdrop. It stays transparent when the
  chrome material is intentionally active. Linux and macOS chrome behavior is
  unchanged.

### Changed

- Refreshed `ARCHITECTURE.md` and `README.md` for the shipped dual terminal
  stack: pinned `libghostty-vt` by default on Linux and Windows x64 MSVC, with
  Alacritty on macOS and as the explicit cross-platform rollback.
- Finalized the Build Week implementation evidence and removed the stale French
  duplicate so the English document is the canonical public account.

No configuration, session format, backend selection, or packaging contract
changed in this patch.

## [0.8.1] - 2026-07-21

### Changed

- Windows 10 1809+ and Windows 11 x64 builds now select the statically linked
  Ghostty terminal engine for new sessions when `terminal.backend` is `auto`.
  Explicit `ghostty` remains available for diagnosis and `alacritty` remains
  the immediate rollback. A Ghostty startup failure can fall back once before
  child spawn; live sessions never switch backend. Linux keeps its existing
  Ghostty selection, macOS remains on Alacritty, and persisted workspace and
  session formats are unchanged.

## [0.8.0] - 2026-07-18

### Changed

- **Ghostty becomes the Linux terminal engine.** New terminal sessions in
  standard Linux builds now run on a pinned, statically linked `libghostty-vt`
  engine: parsing, terminal state, input, snapshots, search, selection, mouse
  reporting, clipboard, hyperlinks, prompt marks, scrollback, resize, and
  shutdown. Alacritty remains available with `terminal.backend = "alacritty"`,
  applied to new sessions only. macOS and Windows stay on Alacritty in this
  release.
- Terminal scrollback is now process-local and is no longer serialized as raw
  output in `session.json`. Restored workspaces keep their layout, cwd, and
  metadata; historical terminal output starts fresh.
- Replaced the one-cell terminal gutter with a fixed 3 px inset so grid, tabs,
  and scrollbar align consistently.

### Added

- Dedicated FFI, safe-wrapper, engine, and smoke-test crates around
  `libghostty-vt`. Ghostty is pinned to `ae52f97d`, Zig to `0.15.2`, and the
  native ABI to `0.1.0`, with verified headers, bindings, build metadata,
  archives, and third-party notices. Normal development builds need neither a
  Ghostty checkout nor Zig, and official packages have no runtime `libghostty`
  dependency.
- A 130-case Alacritty/Ghostty differential corpus (115 exact-parity cases plus
  15 explicitly pinned semantic differences), parser and snapshot fuzz targets,
  chunk-boundary checks, ABI/layout validation, PTY stress coverage, callback
  panic containment, and release-size gates.
- Previous/next prompt navigation actions backed by OSC 133 prompt marks.
- Reversible 120 ms hover feedback across Settings, pane tabs, Review, Agents,
  broadcast controls, sidebars, custom buttons, terminal search, scrollbars, and
  window controls; fixed navigation headers for Agents, Changes, and Settings.

### Removed

- The experimental Rosetta status surface, its title-bar toggle, runtime state,
  runbook, and the `rosetta_enabled` / `rosetta_show_passive` settings. Agent
  attention remains available through the sidebar, desktop notifications, tab
  indicators, and the Attention Queue.

## [0.7.11] - 2026-07-14

### Changed

- Rebuilt workspace cards around a compact title row plus a single metadata row
  for Git branch, changed files, and detected services, with multi-agent state
  consolidated into one prioritized indicator.
- Bundled JetBrainsMono Nerd Font Mono (regular through bold, plus italics) and
  made it the cross-platform terminal default, with built-in Nerd Font glyph
  coverage for shells and prompts.
- SGR bold now advances one weight from the configured base instead of forcing
  weight 700.
- Updated the pinned GPUI/Zed fork to a newer upstream base while retaining the
  Paneflow streamed-Markdown append optimization.

### Added

- A persistent completion indicator that stays visible until the workspace is
  opened, and directly clickable detected frontend services on workspace cards.

### Removed

- The experimental Hera shadow-terminal integration, diagnostic side-by-side
  renderer, and dogfood crates.

## [0.7.10] - 2026-07-10

### Fixed

- Long workspace titles no longer overflow their sidebar cards; the
  active-session indicator stays visible and rename fields stay contained.

## [0.7.9] - 2026-07-07

### Fixed

- Corrected the Windows MSI publisher pin to `StriveX`, fixing the misleading
  "corrupt or tampered" self-update error introduced in 0.7.7. Added a release
  smoke check comparing the pinned publisher against the Authenticode subject,
  and made GitHub Releases draft-only until every signed asset is verified.

## [0.7.8] - 2026-07-07

### Added

- Three branded dark theme presets - `Vercel`, `Claude`, and `Cursor` - with
  their own UI tokens, terminal palettes, and syntax colors. Presets can define
  app-wide UI colors, not just terminal ANSI slots.
- Bundled `Geist` and `Geist Mono` (SIL OFL). The UI font defaults to `Geist`
  and the terminal font to `Geist Mono` on every platform.

### Fixed

- Windows: external URLs, editor launches, folder opens, taskbar identity
  (`Strivex.PaneFlow`), Start Menu shortcut icons, and notifications were
  hardened for packaged installs.
- Terminal hyperlink hover no longer paints a full URL tooltip, and sidebar
  diff stats hide zero-count sides instead of showing `+0 -0`.

## [0.7.7] - 2026-07-06

### Changed

- Broad hardening pass across the agent control plane, terminal runtime,
  workspace persistence, diff workflow, and distribution pipeline (36 commits).
- MCP pane tools now advertise read-only behavior, clamp output, default to the
  current workspace, and require an explicit opt-in for instance-wide reads.
  Terminal data returned to agents is fenced as untrusted output.
- `workspace.up` plans pane spawns before execution, canonicalizes working
  directories, dedupes labels, and bounds context/prefill content.
- Notification surfaces are now **opt-in** instead of on by default.
- `surface.send_keystroke` rejects newline bytes; use `surface.send_text` for
  text submission.
- Persisted surface schema exposes tabbed surface fields so split/tab layouts
  restore faithfully.
- Signed self-update flows reject unsigned or keyless builds before touching
  disk; Windows MSI updates stage through a relay after the process exits.

## [0.7.6] - 2026-07-01

### Added

- Settings → Terminal now exposes cursor shape, cursor color, font size, font
  weight, line height, cell width, integrated block glyphs, color emoji,
  ligatures, scrollback, and Windows terminal material, with matching
  `paneflow.json` schema coverage.

### Changed

- Renderer pass: text rounds onto stable cell pixels, cursor painting handles
  wide glyphs and emoji accurately, and the default cursor color moves to Apple
  system blue. Golden snapshots refreshed.
- Windows-only terminal and chrome material toggles let Mica or blur show
  through, while non-CLI surfaces stay opaque and cross-platform configs stay
  safe on Linux/macOS.
- Pinned Rust toolchain bumped to 1.96.1 across local, CI, release, and docs.

### Security

- Updated `anyhow` to clear `RUSTSEC-2026-0190`.

## [0.7.5] - 2026-06-30

### Added

- Agent session discovery and resume support for Pi, Hermes, Grok, Cursor
  Agent, Gemini, and Kiro, on top of Claude, Codex, and OpenCode. Resume keeps
  the strict session-id guard before anything reaches a PTY.
- Restart controls for bottom-dock and sidebar agent terminals so they pick up
  the active integrated-terminal shell.
- Files sidebar animation, focus-on-toggle, and arrow/Home/End/Enter/Escape
  keyboard navigation.
- Diff review interactivity: unified/split switching, collapse-all and per-file
  folding, fold expansion, per-file horizontal scrolling, and language icons.

### Changed

- Native compositor backdrop/blur on Linux is now opt-in through
  `PANEFLOW_LINUX_NATIVE_BACKDROP`; Wayland/X11 users get an opaque background
  by default.

## [0.7.4] - 2026-06-29

### Added

- A visual workspace-template builder in Settings, backed by
  `commands[].workspace`: project path, layout preset, panes, agents, shell
  commands, pane labels, cwd overrides, env, and prompt prefill, launched
  through the same `workspace.up` path as the CLI.
- A workspace context-menu "Run Workflow" action that launches a matching saved
  template into an already-open workspace.

### Changed

- The Terminal settings page keeps the primary controls (cursor shape, bell,
  font family, font size); advanced knobs remain available in `paneflow.json`.
- Windows and macOS chrome material stays unobscured; Linux keeps its
  readability veil.

## [0.7.3] - 2026-06-29

### Fixed

- Windows hook commands are wrapped in an explicit PowerShell invocation, so
  installed paths under `C:\Program Files\PaneFlow\...` work when a hook runner
  executes through PowerShell instead of `cmd.exe`. Applies to Codex's
  `commandWindows` field, the Claude-compatible strict configs (CodeBuddy,
  Qoder, Gemini, Cursor, Grok, Hermes), and persistent
  `paneflow hooks setup` entries.

## [0.7.2] - 2026-06-29

### Fixed

- Codex hook commands on Windows when Paneflow is installed under a path with
  spaces, including `C:\Program Files\PaneFlow`.
- Paneflow-managed hook cleanup for PowerShell-wrapped commands, including
  quoted paths and orphaned hook commands.
- The sidebar update banner: clicking the banner starts the self-update flow,
  while the dismiss control only dismisses.

## [0.7.1] - 2026-06-28

### Fixed

- Agent status hooks on Windows for release installs under paths such as
  `C:\Program Files\PaneFlow`, including Codex fallback commands that no longer
  rely on POSIX single quotes.
- Gemini hook installation updated to the current matcher-group schema and
  millisecond timeout; Cursor kept on its flat `hooks.json` schema with a
  Windows-safe command string.

### Changed

- Rewrote `README.md` around the current native Rust/GPUI release surface,
  reworked `ABOUT.md`, added `llms.txt`, and refreshed `ARCHITECTURE.md` to
  match the codebase.

## [0.7.0] - 2026-06-28

### Added

- **Rosetta**, an in-app top-center agent status surface for CLI and Agents
  mode. It derives rows from workspace `agent_sessions`, Agents-mode thread
  status, surface ids, and waiting timestamps, then groups them into
  needs-input, failed, stalled, running, and recent sections. Rows are
  actionable and route back to the originating pane or thread.
- A dedicated Settings → Notifications page covering native OS notifications,
  Rosetta visibility, and passive running-agent summaries, backed by
  `rosetta_enabled` and `rosetta_show_passive`.
- JSON tool manifests under `mcps/paneflow/tools/` for `list_panes`,
  `read_pane`, and `search_pane`, carrying the untrusted-output warning.

### Security

- Agent-provided text in Rosetta is treated as untrusted data: plain text only,
  capped, sanitized, and visually separated from Paneflow controls. Freeform
  agent messages cannot create approval buttons.

## [0.6.10] - 2026-06-27

### Fixed

- Windows in-app updates now use a detached relay that survives the main
  process exit, moves UAC elevation to the relay's `msiexec` invocation, waits
  for the installer to finish, and relaunches Paneflow on success, cancel, or
  failure. The staged MSI is always removed afterwards.

### Changed

- Release publishing is now gated by every real validation job: all Linux
  distro smokes, a new end-to-end Windows MSI install/relay/uninstall smoke on
  `windows-2022`, and the Linux auto-update E2E. The auto-update E2E signs its
  own tarball so it stays a true pre-publish gate.

## [0.6.9] - 2026-06-27

### Fixed

- Windows agent hooks now use a native `commandWindows` entry invoking
  `paneflow-ai-hook.exe` through `cmd.exe /D /C`, so Codex and compatible
  agents no longer route Paneflow hooks through bash or WSL. This clears the
  repeated `SessionStart` / `PreToolUse` / `PostToolUse` `hook exited with
  code 1` failures on Windows machines without a WSL distribution. The portable
  `command` field is still written as the cross-platform fallback and
  ownership marker.

## [0.6.8] - 2026-06-27

### Fixed

- Windows MSI self-update now uses a native relay staged as
  `paneflow-msi-relay-<pid>.exe` in `%TEMP%` instead of a hidden PowerShell
  handoff, behind a hidden `--msi-relay` entrypoint that runs before GPUI and
  single-instance handling. Machine-wide installs request UAC **before**
  Paneflow exits, so cancelling elevation leaves the app open. Failed updates
  now leave diagnostics in `%TEMP%`.

### Upgrade note

- Users on 0.6.6 or 0.6.7 whose in-app updater closes Paneflow without
  installing must install 0.6.8 manually from the MSI once; the broken handoff
  lives in the old binary.

## [0.6.7] - 2026-06-27

### Added

- Restored native desktop notifications for agent lifecycle events - completed
  turns, approval/input requests, unexpected exits, and stalled agents - when
  Paneflow is not foregrounded.
- An animated startup splash, smooth sidebar transitions, fixed-width terminal
  tabs, hover-to-close tabs, better title truncation and tooltips, and a
  collapsible tab-bar action cluster.

### Changed

- Service badges are less timing-dependent, Windows now participates in port
  detection, and agent status is shared between the UI and IPC.

### Known issue

- The built-in updater in 0.6.7 cannot correctly upgrade to newer releases on
  Windows; install the newer MSI manually once. Fixed in 0.6.10.

## [0.6.6] - 2026-06-26

### Fixed

- The Windows MSI updater no longer asks Windows Installer to replace
  `paneflow.exe` while Paneflow is alive. It downloads and verifies the MSI,
  saves the session, starts a detached relay, then exits; the relay waits for
  the GUI process to disappear, runs `msiexec`, removes the temporary MSI, and
  relaunches. This avoids the native `FilesInUse` dialog.
- APT updates now install `paneflow=<version>-1`, matching the `.deb` version
  emitted by `cargo-deb`. DNF stays on `paneflow-<version>`.
- The legacy `.run`/tarball migration path is now Linux-only, so macOS
  `Unknown` installs no longer write a Linux-style `~/.local/paneflow.app`
  layout.

## [0.6.5] - 2026-06-26

### Fixed

- Windows terminal panes track shell cwd reliably through OSC 7 integration for
  PowerShell 5.1, PowerShell 7, and Git Bash, captured before Alacritty
  consumes the stream. Shell preset matching no longer mislabels
  `C:\Windows\System32\bash.exe` as Git Bash.

### Changed

- Review and Agents diff views window large split diffs around hunks, retain
  diff rows across mode switches, and filter duplicated worktree columns.
- Windows is now a hard release target: the signed MSI build, packaging, and
  verification must pass before a stable release is published.

## [0.6.4] - 2026-06-25

Supersedes 0.6.3, which shipped without a complete Windows MSI artifact.

### Fixed

- Windows environments where launching `claude`, `codex`, `opencode`, `gemini`,
  and other helper shims could be blocked by Application Control / AppLocker /
  Smart App Control. The MSI now installs signed helper binaries under the
  managed install directory instead of relying on per-user extraction from
  `%LocalAppData%`.
- `${port_offset}` support inside `flow.toml` pane environment variables.
  `flow.toml` now shares port allocation behavior with `paneflow up`, including
  port-base handling and busy-port skipping, and supports flow-level
  `port_base`. `flow --dry-run` includes resolved pane environment data.

## [0.6.3] - 2026-06-25

Superseded by 0.6.4, which shipped the complete Windows MSI artifact.

## [0.6.2] - 2026-06-24

Patch release focused on Windows trust and cleaner agent terminal startup.

### Changed

- Windows release builds now Authenticode-sign `paneflow.exe` before packaging
  the MSI, then sign the MSI as well. This makes the installed executable
  verifiable by Smart App Control instead of relying only on installer trust.
- Agent panes spawned through `paneflow up`, `paneflow flow`, and IPC now carry
  an explicit terminal surface profile into the app.
- PowerShell agent panes now start with `-NoProfile`, avoiding user profile
  startup noise while preserving normal PowerShell behavior for regular panes.

## [0.6.1] - 2026-06-24

Patch release focused on keeping long-running multi-agent work leaner and less
surprising after the Paneflow Conductor release. It also closes two macOS/UI
paper cuts from the issue #11 feedback loop and polishes toast feedback across
the app.

### Added

- Memory-oriented terminal surface profiles. Normal terminals keep the existing
  10,000-line scrollback default, while agent, Review, and cold cached terminal
  surfaces are capped at 4,000, 2,000, and 1,000 lines respectively.
- A memory smoke-test runbook covering the 6-8 agent workload, Review and
  Agents diff navigation, IPC bursts, and the OS verification gaps that must be
  called out explicitly.

### Changed

- Multi-agent retention is now structurally bounded instead of relying on
  process memory staying friendly. Agent terminal hot cache, bottom terminal
  retention, session sidebar rows, attribution matches, closed-pane scrollback,
  diff rows, raw diff file reads, and GPUI-bound IPC requests now have explicit
  caps.
- Hidden Review columns and closed Agents diff panels release their loaded row
  models, display caches, offsets, attribution data, and exited review-terminal
  references. Running review terminals are protected: Paneflow asks you to close
  them before hiding the column instead of silently killing work.
- IPC requests headed for the GPUI thread now use a bounded queue with
  backpressure, and each UI tick drains a bounded number of live and cancelled
  requests. Busy clients get a clear retryable overload error instead of
  letting request memory grow without a cap.
- Notification toasts use icon assets, tighter sizing, clearer action buttons,
  and error-style detection for failure messages.

### Fixed

- The sidebar's CLI / Review / Agents switch now stays visible as persistent
  primary navigation, with Settings moved to a compact utility button. Switching
  modes also closes the Settings popover so the footer state stays predictable.
- macOS GUI launches that inherit the filesystem root no longer create a fresh
  implicit workspace at `/` with a generic `Terminal 1` label. New implicit
  launches fall back to the home directory, and legacy restored `Terminal N`
  root workspaces are repaired without affecting explicitly requested root
  terminals.
- macOS native menu items stay enabled across CLI, Review, and Agents modes by
  registering app-global fallback handlers for the menu actions.
- Linux shim size budgets were rebaselined after the release-min helper binary
  grew from the conductor work, keeping CI focused on real size regressions.

## [0.6.0] - 2026-06-21

Paneflow Conductor. Paneflow becomes a control plane for a fleet of CLI coding
agents running side by side in panes. A new public `paneflow` CLI discovers,
reads, drives, and waits on those agents over the local IPC socket, never by
scraping the screen, and a harness-agnostic conductor SKILL lets any agent
(Claude Code, Codex, OpenCode, Gemini, ...) drive the others. Cross-platform:
Linux, macOS, and Windows over the named-pipe transport.

### Added

- The `paneflow` fleet CLI. `ps` / `ls` discover the running agents and the
  panes themselves; `status` / `read` / `search` inspect one agent's live state
  and scrollback; `send` dispatches a prompt (pre-filled by default, `--submit`
  to auto-submit, `--broadcast` to fan out across matching panes); `wait` and
  `watch` block on events instead of polling; `up` spawns agents from a
  declarative `paneflow.workspace.toml`; and `flow run` executes a declarative
  spawn -> wait -> feed -> review pipeline. Target any pane by `surface_id`,
  name, `cmdline:<substr>`, or `cwd:<path>`. The MCP tool names (`list_panes` /
  `read_pane` / `search_pane`) are accepted as aliases, and a genuinely unknown
  verb exits non-zero with an actionable error instead of opening a stray GUI
  window.
- Hooked agents with an authoritative state machine. Agents launched through
  `up` / `flow` (or carrying a hook integration) are tracked turn by turn: their
  `state` (`thinking`, `waiting_for_input`, `finished`, `errored`, `stalled`,
  `idle`, or `unknown_running`) is real, their `ai.stop` / `ai.notification`
  events fire, and their `last_result` is exposed. A self-diagnosing hook
  installer reports `hooked` plus a `reason` in the fleet, so an agent that was
  only spotted by a process scan reads `unknown_running` rather than faking a
  derived state.
- An outbound IPC event bus. `events.subscribe` streams `ai.*` transitions and
  surface changes, which `paneflow watch` renders as one JSON event per line
  with a 30s heartbeat. `wait --idle` blocks on output quiescence via the
  monotonic `output_generation` counter; `wait --pattern` blocks on a
  baseline-aware sentinel (it matches output produced after the wait starts, not
  the prompt echo); `--all` / `--any` gate across many panes at once.
- Deterministic auto-submit. `send --submit` wraps the prompt in bracketed paste
  and sends the carriage return as a separate, calibrated write, then confirms a
  hooked agent's turn actually started via `output_generation`. If no turn start
  is confirmed it exits non-zero instead of returning a false `submitted:true`.
- Structured inter-agent context. A turn's `last_result` is backfilled
  off-thread from the Stop-hook transcript, so a full-screen (alt-screen) TUI's
  report is captured in full rather than truncated to the viewport, and
  `send --report-file <path>` appends a precise file contract so an agent writes
  its complete report to disk and prints `REPORT_DONE <path>`.
- AI access controls under Settings > AI Agent. An `ai_unrestricted` free-access
  toggle sanctions CLI auto-submit and traced writes; an `ai_injection_fence`
  wraps `read` output in an `<untrusted_terminal_output>` fence (drop it with
  `read --raw` only when you trust the source). Writes are refused with a clear,
  actionable error unless free access is on or `PANEFLOW_IPC_SCRIPTING=1` is set.
- The harness-agnostic **Paneflow conductor** SKILL: a shell-only playbook for
  driving the fleet (discover -> read -> wait -> dispatch -> hand back), plus a
  committed `examples/review-pipeline.flow.toml` worked cross-vendor
  implement -> review pipeline to copy from.

### Changed

- The event bus streams over Windows named pipes through a `PeekNamedPipe`
  liveness guard, falling back to a bounded `output_generation` clock where a
  transport cannot tick subscriptions. The `paneflow` client resolves the
  build-profile socket path and honors `PANEFLOW_SOCKET_PATH`.
- The inter-agent context blob is written owner-only (0600/0700). The
  Thinking-stuck watchdog tightened to 60s, and a pane's label is applied
  atomically at spawn so it never flashes the generic agent name first.
- Em dashes were normalized to ASCII hyphens across the repo. A new `ABOUT.md`
  project overview landed, and the fleet / events / flow control-plane surface is
  documented under `docs/`.

### Fixed

- macOS: the parent-death reaper is guarded against PID reuse, and the shim adds
  an orphan guard (Windows gets a Ctrl+C `ai.stop`).
- A stale persistent hook is now ignored in favor of a project-local install.
- The Agents environment toolbar no longer covers the embedded CLI (a top band
  is reserved), and per-file horizontal scroll is restored in the diff body.
- Windows: the console-subsystem binary is kept (only the lonely GUI console is
  shed), the workspace test suite passes natively, and the macOS DMG and process
  unit tests are made cross-platform.

## [0.5.10] - 2026-06-19

### Fixed

- Per-file horizontal scrolling is back in the diff body. Each file in the
  Agents diff dock and the Review column owns its own horizontal offset again
  after the shared direct-paint `DiffElement` migration, reachable with
  Shift+wheel or a trackpad horizontal swipe while native vertical scrolling
  stays untouched.
- The Agents environment toolbar no longer covers the CLI: the model selector
  and layout toggles sit in a reserved top band, so narrow windows cannot make
  the floating toolbar paint over the first terminal lines.

### Changed

- Added shared diff horizontal-scroll geometry in `diff/hscroll.rs`, reused by
  both the Agents dock and the Review column, with per-file span metadata
  computed off the render path.

## [0.5.9] - 2026-06-18

A review-workflow release. The Agents diff dock and the Review view now render
through one shared diff pipeline, the review loop is fully keyboard-driven, and
the Review attribution badge can show which agent wrote a change alongside an
estimated, fully local token cost.

### Added

- Keyboard-first review loop. `]` / `[` jump between hunks, `u` toggles the
  unified/split view, `s` toggles cross-column scroll sync, `Esc` dismisses,
  and `a` acts on the focused hunk. Bindings are scoped to
  `DiffView && !Terminal && !TextInput` so an embedded review or shell terminal
  and the base-branch filter input keep their own keystrokes, and they are
  remappable through the action registry.
- Per-hunk act-on-hunk actions in the Review view, with prompts pre-filled into
  a freshly launched review CLI rather than auto-submitted.
- Agent attribution and estimated cost on the Review badge. Per-session token
  usage is folded across assistant turns by the Claude Code, Codex, and
  OpenCode scanners, and a build-time-embedded, versioned pricing table turns
  it into an estimated cost. It is 100% local with no network lookup; unknown
  models show their token counts with no fabricated cost.
- A `review_prefill_delay_ms` setting (default 2000 ms, clamped to
  [250, 10000]) with a `-` / `+` stepper under Settings > AI Agent > Review,
  tuning how long Paneflow waits before auto-typing a prompt into a freshly
  launched review CLI. The clipboard fallback keeps any value safe.

### Changed

- The Agents diff dock now renders through the same `DiffElement`, git pipeline,
  and row model as the Review view. The bespoke horizontal-scroll state was
  replaced by a single shared scroll handle, and the monolithic diff view was
  split into focused submodules (loader, scroller, interaction, review,
  attribution, render).
- New-chat thread titles are now derived from the on-disk session ai-title
  instead of staying on the generic agent label. Each Claude thread is bound to
  a forced `claude --session-id <uuid>` minted at creation so it maps 1:1 to its
  session file (resuming the same id appends, so a restart continues the same
  session); at turn end the polished ai-title is backfilled into the sidebar row
  off the main thread. A manual rename locks the title against later OSC updates
  and backfills. Every session id is re-validated before it reaches the command
  line, so a tampered `session.json` cannot inject an argument.

## [0.5.8] - 2026-06-17

An Agents sidebar cleanup. Thread status is now driven solely by the agent
hook lifecycle (Claude Code / Codex shims), removing the output-activity
heuristic that lit a "thinking" spinner from raw PTY traffic and produced
false positives. The environment panel also sits flush against the right edge.

### Changed

- Agents thread status now comes only from `ai.*` hook frames. The fallback
  heuristic that inferred "thinking" from PTY output bursts (for agents without
  a hook integration, such as OpenCode, Pi, and Hermes) is gone: it lit false
  spinners on dev-server output streaming under a bare-shell thread and on TUI
  redraws, and never matched the precise hook lifecycle that the Claude Code and
  Codex shims already provide.

### Fixed

- The Agents environment panel now sits flush against the right edge, tightened
  from a 38px to a 12px inset now that nothing reserves the gutter.

## [0.5.7] - 2026-06-17

A macOS reliability pass. The headline is the DMG self-updater, which froze
on every attempt because the codesign Team-ID pin silently failed; this also
relights the workspace agent dot, resolves bare configured shells under a GUI
launch, and stops a spurious "shell may have exited" warning. The pid-0 guard
lands on Linux too; the rest is macOS-only or dev-only.

### Fixed

- The DMG self-updater no longer freezes on macOS. The codesign Team-ID pin
  passed its requirement as a separate `-R <req>` argument, which macOS 15+/26
  read as a *file path*: codesign tried to open the inline requirement text as
  a file and aborted, so every DMG update failed and the updater stalled at the
  three-strikes "Update keeps failing" toast. The requirement now uses the
  attached `-R=<req>` form (a single argv element) that every supported macOS
  parses as inline requirement source.
- The workspace card now lights its agent dot on macOS again. `proc_listchildpids`
  returns zero children for an unprivileged caller on modern macOS, so the
  per-node subtree walk found nothing. Agent detection now builds a
  parent-to-children map from `proc_bsdinfo.pbi_ppid` once per scan and walks
  it breadth-first, mirroring the existing Linux fallback.
- A bare configured shell name (e.g. `"pwsh"`) is now resolved under a GUI
  launch whose inherited PATH omits `/opt/homebrew/bin`, instead of silently
  falling back to `/bin/sh`. After the PATH search misses, Paneflow probes the
  well-known Unix install dirs (Homebrew prefixes and system dirs), the macOS
  parallel to the Windows well-known-location probe.
- Display-only terminals no longer probe a bogus process. A display-only pane
  has no real PTY (`child_pid == 0`); on Linux that meant reading `/proc/0/cwd`,
  and on macOS `proc_pidinfo(0, …)` targeted the kernel swapper, failed with
  EPERM, and spammed a misleading "shell may have exited" warning on every poll
  tick. The cwd probe now bails before the syscall, matching the existing
  foreground-command guards on every platform.
- Debug builds no longer warn about running outside a `.app` bundle. A
  `target/debug/` binary is never inside a bundle (the expected dev path), so
  that message is now logged at debug level in debug builds; release binaries
  running outside a bundle still warn, since that is a genuine ad-hoc extraction
  worth surfacing.

## [0.5.6] - 2026-06-16

The Agents git diff dock becomes resizable and scrolls each file on its own,
and every diff surface in the app now draws from one color source.
Cross-platform.

### Added

- The Agents git diff dock is now resizable: drag its left edge to widen or
  narrow it. The width is clamped between a readable floor and a
  window-friendly ceiling so the dock can never swallow the terminal column or
  shrink below a usable code width.
- Each file in the Agents diff dock now scrolls horizontally on its own, so a
  long line in one file no longer drags the short files into the blank. Files
  that overflow grow a per-file horizontal scrollbar (click the track or drag
  the thumb) and accept horizontal wheel scrolling, while vertical scrolling
  stays shared across the dock.

### Changed

- Every diff surface now reads its +/- colors from a single shared source:
  Codex green/red on dark themes, the theme's version-control colors on light
  themes. The Agents diff dock, the Diff/Review view, the CLI workspace sidebar
  diffstat and the diff sidebar previously each inlined their own hex and could
  drift apart; they are now guaranteed to match.
- The Agents environment toolbar's editor split-button now shares the rounded
  radius of the toggle buttons beside it.

## [0.5.5] - 2026-06-16

The Agents view gains a Codex-style environment surface, and the CLI tab strip
is restyled to match it. Cross-platform.

### Added

- An environment card in the Agents view. It carries a per-repository git branch
  picker (a live-filtered, focus-trapped search field that also names a new
  branch) and an external-editor selector that reuses the same editor list and
  logos as the General settings tab. The card is scoped to the active thread's
  working directory, so project threads and free chats can each point at their
  own repository.
- A full-width bottom terminal dock in the Agents view, toggled from the
  environment toolbar. It hosts a tab strip of shell terminals: open as many as
  you like with `+`, close each one independently, and drag the dock's top edge
  to resize it. Every terminal is a real PTY whose scrollback and I/O survive tab
  switches and closing or reopening the dock, so coming back is always warm.
- A right-side git diff dock in the Agents view. It renders an off-thread diff
  snapshot for the active thread's working directory, with a unified or
  side-by-side split view, per-file fold state that survives re-renders, and an
  uncommitted-files count surfaced from `git diff --shortstat`.

### Changed

- The CLI multiplexer's terminal tabs are now floating, rounded chips instead of
  full-height bordered tabs. The active chip lifts on a whisper of the text
  color, inactive chips wash in on hover, and the chrome separators are gone, so
  the strip melts into the terminal body and speaks the same tab language as the
  new Agents bottom dock.

## [0.5.4] - 2026-06-16

A visual polish pass on the app chrome plus two Windows session fixes. The
chrome refresh lands on every platform; the session and title-bar fixes are
Windows-only.

### Fixed

- The agent-sessions sidebar now populates on Windows. Claude Code, Codex and
  opencode sessions for the open workspace were never listed because three
  things were wrong at once: the project-directory slug kept the drive
  letter's `:` (so `C:\dev\paneflow` looked for `C:-dev-paneflow` instead of
  the real `C--dev-paneflow`), the working-directory filter was case- and
  separator-sensitive, and the active terminal's cwd was never seeded on
  Windows. All three are fixed, so the sidebar resolves the same sessions your
  agent CLIs actually wrote.
- Terminal tabs and Agents threads no longer take the shell's own path as
  their name on Windows. PowerShell and cmd briefly title their window with
  their executable path (e.g. `C:\Program Files\PowerShell\7\pwsh.exe`) before
  your profile runs; PaneFlow now ignores a title that is merely a path to an
  `.exe` and keeps the real label.

### Changed

- A chrome refresh across the sidebars, title bar, context menus and settings.
  Hovered and selected rows now share one slightly brighter translucent
  material (closer to Codex/OpenAI's soft highlights), drop-shadows are gone
  for a flatter look, and the docked sessions and files rails use the same
  native window material as the rest of the app instead of a flat dark fill.
  Corner radii are unified across cards, rows and settings controls.
- Quieter logs. A failed update check from a transient network or GitHub
  hiccup, and a diff column superseded by a newer load, now log at debug
  instead of warn; only an actionable update failure (a persistent 4xx) still
  warns.

## [0.5.3] - 2026-06-15

A Windows quality pass: new terminals now open in the right directory, the
font picker is wired end-to-end, and two stray-window/log annoyances are gone.
No changes on Linux or macOS.

### Added

- Font picker on Windows. The Settings font list was empty on Windows because
  family enumeration was never implemented; it now enumerates installed
  fixed-pitch families via GDI (`EnumFontFamiliesExW`), alongside the fonts
  PaneFlow embeds. GDI is used only for discovery; GPUI/DirectWrite still does
  the rendering.
- Cascadia Mono as the Windows default font. A fresh install now defaults to
  the system Cascadia Mono, matching Windows Terminal, instead of the embedded
  IBM Plex Mono. Linux and macOS still default to the embedded mono, which also
  stays available everywhere as the fallback. Pick any installed font (or
  return to the default) from the Settings list.

### Changed

- The font-family picker moved from the Themes page to the Terminal page, next
  to font size, line height and ligatures. Searching "font" in Settings now
  jumps to the Terminal page, and the Themes page is theme-only.

### Fixed

- New terminals open in the workspace directory on Windows. Opening a new tab,
  splitting a pane, or duplicating a tab spawned the shell in
  `C:\Program Files\PaneFlow` (the install directory) instead of the project
  folder, because Windows can't introspect a child process's working
  directory. New panes now fall back to the workspace's own root, so every new
  terminal lands where you'd expect.
- No more console window flashing on Windows. Background helpers PaneFlow runs
  (git status polling, agent CLIs, MCP probes) each briefly popped an empty
  console window; they now spawn with `CREATE_NO_WINDOW`.
- No more spurious warning when a Windows shell closes. Typing `exit` logged a
  harmless-but-noisy `TerminateProcess failed` warning on every shell close;
  PaneFlow now detects the already-exited child and skips the kill path.

## [0.5.2] - 2026-06-15

A Windows hotfix: the in-app updater now works on MSI installs. No changes on
Linux or macOS.

### Fixed

- Windows self-update. Clicking "Update" on an MSI install failed with "HOME
  environment variable is not set" and never updated. The running binary's
  install location was misdetected - `std::fs::canonicalize` returns the
  extended-length `\\?\C:\…` path on Windows, which did not match
  `%ProgramFiles%`, so the install was classified as unknown and the updater
  fell back to the Linux tar.gz path (which reads `$HOME`). MSI installs are
  now detected correctly and the update runs through msiexec end-to-end. As a
  safety net, an unknown install on Windows no longer routes to the Linux
  updater either.

  Note: because the currently-running build carries the old, broken detection,
  it cannot self-update to this fix - install the 0.5.2 `.msi` manually once
  from the releases page, and the in-app updater will work for every release
  after it.

## [0.5.1] - 2026-06-15

A Windows polish patch on top of 0.5.0: the app and installer now carry the
right icon, and the stray console window is gone. No changes on Linux or macOS.

### Fixed

- No more stray console window on Windows. paneflow.exe is now built as a
  GUI-subsystem binary, so launching it from Explorer, a shortcut or the Start
  Menu no longer opens an empty extra terminal window beside the app. The
  scriptable CLI (paneflow mcp install, paneflow ls, --version, …) still works:
  the process re-attaches to the parent console when started from a terminal.
- The paneflow.exe icon in Explorer. The bare executable embedded no Windows
  resource and fell back to the generic Windows icon; it now ships the same
  multi-resolution PaneFlow icon as the installer.
- The Windows installer icon. The 0.5.0 MSI still showed the old logo on its
  Start Menu shortcut and Add-or-Remove-Programs entry - the WiX icon was the
  one output the new-logo regeneration had missed. It is now regenerated from
  the new logo, and the icon pipeline mirrors it on every run so it can no
  longer go stale.

### Documentation

- Refreshed the Windows install docs for the signed v0.5.0 .msi: the native
  installer is now documented as an available path (WSL2 kept as the
  alternative), with a SmartScreen "Run anyway" walkthrough (publisher:
  StriveX) and signature-verification steps, replacing the stale "no native
  build / Q3 2026" framing across the docs.

## [0.5.0] - 2026-06-15

This release brings Paneflow to Windows and lands a ground-up redesign of the
app shell.

### Added

- Windows support. Paneflow now runs on Windows 10 and 11. The title bar
  carries native Windows 11 caption buttons and a full-width inset panel, new
  terminals default to PowerShell, and live agent-status updates are delivered
  reliably over named pipes.
- Inline settings. The settings window is replaced by a Codex-style settings
  surface embedded directly in the app, built on a shared set of select,
  toggle and card primitives, with every page rebuilt on those controls.
- The PaneFlow Light theme returns, paired with a light app shell, and the
  window backdrop now seeds itself from the active theme mode.
- Configurable font fallbacks. A user-editable font_fallbacks list lets you
  control the monospace fallback chain.

### Changed

- Cockpit chrome redesign. A reworked window chrome with a native backdrop,
  title-bar Files and Help menus, a Profile menu, and a sidebar toggle. The
  title bar now spans the full window width on every desktop platform.
- One menu language across the app. The title-bar dropdowns, the workspace and
  agents context menus, the theme picker, and the diff scope, project, branch
  and base pickers all share a single elevated surface and select-row style.
- The agent launcher is laid out as a grid of filled tiles, and the agents
  sidebar search field matches the settings search pill.
- The About dialog is restyled as a native app dialog, and hover backgrounds
  align with the active selected state.
- The option-as-meta default is now platform specific.

### Fixed

- Self-update reliability across platforms: the macOS app bundle relaunches
  correctly and handles translocation, AppImage installs are detected via
  $APPIMAGE with the right package-manager routing, the Fedora upgrade path
  refreshes its metadata first, and a mismatched-signature install surfaces a
  clearer hint.
- Terminal teardown is guarded against PID reuse and works on kernels built
  without CONFIG_PROC_CHILDREN.
- The GUI now adopts the login-shell PATH on launch, so tools on your shell
  PATH are found when Paneflow is started from a launcher.
- Turn-end desktop notifications carry the Paneflow icon, and widget text
  keybindings are re-registered on every keymap apply.
- Linux packages depend on fontconfig so the settings font picker is
  populated.

## [0.4.4] - 2026-06-11

### Changed

- The in-pane find bar is now a real editable field. It hosts the same text
  input the agent sidebar uses, so opening a search puts a live caret in the
  field with selection, IME and clipboard support, and the query updates the
  match list as you type. Its chrome follows the active theme (One Dark /
  PaneFlow Light) instead of a fixed palette, with search, regex, fleet,
  previous, next and close controls, and a status line that reads the match
  position, an empty result, or an invalid pattern.
- Every agent other than Claude Code now shows the same rotating arc the agent
  sidebar uses while it is thinking, in a soft neutral grey, replacing the
  Codex-style pulsing dots. Claude Code keeps its own glyph spinner and salmon
  identity colour.

## [0.4.3] - 2026-06-11

### Added

- Composer: a bottom-anchored multi-line input (secondary-shift-space) over
  the focused pane. Enter pre-fills the agent through bracketed paste
  without submitting, so the prompt is yours to review before it is sent;
  secondary-enter pre-fills and submits in one keystroke.
- Named broadcast groups: assign panes to a group (secondary-shift-b to
  toggle membership, secondary-shift-m for the picker), each marked by a
  3px coloured edge stripe. The Composer fans one prompt out to every live
  member of the active group and shows a transient recap of who received
  it, so a broadcast is never silent.
- Queued prompts for busy agents: a prompt sent to a generating agent is
  held ("1 queued" tab chip) and flushed automatically on that session's
  next idle transition, instead of being dropped or spliced into the
  running turn.
- Attention Queue (secondary-shift-k): a single overlay listing every agent
  waiting for input across all workspaces, with its question and how long
  it has waited, longest-waiting first. Enter warps straight to that pane.
- Launch Pad (secondary-shift-l): worktree, split, agent launch and
  first-prompt prefill in one gesture.
- Agent status beyond Claude Code and Codex: the sidebar states, tab dots
  and notifications now apply to any agent CLI launched through the shimmed
  PATH, identified by its binary name; an unrecognized tool is reported as
  itself instead of being mislabeled as Claude.
- Scrollbar match rail: an active search projects every match as a tick on
  the scrollbar track (decimated to the pixel grid, so 10 000 matches cost
  the same as 10), with the existing proportional track click to jump.
- Fleet grep: from any pane's find bar, the "Fleet" toggle (or Alt+F) runs
  the same query across every pane of every workspace off the render
  thread, lists the matching panes with counts, flashes a transient match
  badge on their tabs, and Enter teleports with the local search pre-armed.
- Per-pane font zoom: Ctrl+= / Ctrl+- / Ctrl+0 (Cmd on macOS) change the
  focused pane's font size by 1 px steps within 8-32 px, with the PTY grid
  reflowing like a window resize. Persisted per pane across restarts;
  panes without an override keep following the global setting.

- Fleet observability: the port/process scan now attributes results to each
  pane. Tabs show a compact identity pill for the agent CLI running inside
  (PID-detected across 16 known agents, persisted across restarts as a
  dimmed "last known" until confirmed) and per-pane port badges, clickable
  when the port belongs to a frontend dev server. When a pane announces a
  URL whose port is actually owned by another pane, its badge turns into an
  alert naming the owner.

- Errored agent state: when an agent CLI launched through the shimmed PATH
  exits non-zero, its session turns red (tab dot + sidebar badge) and the
  desktop notification says "agent exited (exit N)" instead of a false
  "agent finished". Human interrupts (Ctrl+C, pane close, external kill)
  still count as finished, never as errors.
- Stalled agent detection (on by default): a thinking agent that emits no
  hook activity for 5 minutes is flagged "stalled" in the sidebar, with one
  desktop notification per stall episode. Threshold configurable via
  `agent_stall_threshold_secs`; kill switch via `agent_stall_detection`.

### Changed

- Dev-server detection is now OS-authoritative. A port badge's clickable
  link is derived from the command line of the process that owns the
  socket, so it no longer depends on catching the dev server's banner line
  in the terminal output. The link survives an in-shell restart (nodemon, a
  plain re-run) that re-binds the port, and sustained agent output no longer
  starves the scan that picks up new ports.

### Fixed

- Agent sessions are reaped the moment their pane closes instead of
  lingering up to 30s for the periodic sweep, covering the cases where the
  exit hook never arrives (shim killed, agent started without the shim).
- A recycled process id can no longer keep a finished agent's status alive:
  a session pins its process start time, and a reused pid whose start time
  differs is treated as gone.

## [0.4.2] - 2026-06-10

### Changed

- New logo artwork. Every icon size (16-512, master 1024, .icns, .ico) is
  regenerated with a transparent keyline margin: the squircle body is
  rendered at ~80% of the canvas, the value GNOME and macOS icon grids
  converge on, so the icon no longer renders oversized next to
  spec-compliant peers in the GNOME Shell dash and macOS dock.

## [0.4.1] - 2026-06-10

### Added

- Live activity indicator on Agents thread rows: a row whose agent is
  working shows a Codex-style spinner, driven by the same `ai.*` signals as
  the pane badges.

### Changed

- Agents panel polish: stronger selected-row contrast against the rail, a
  faint hairline between rail and panel, and a 16px panel corner radius
  matching the Cli/Diff silhouette.

## [0.4.0] - 2026-06-10

### Added

- `paneflow` CLI: a scriptable control plane over the IPC socket. `ls`,
  `read` and `search` expose pane scrollback with a unified target selector;
  `new`, `select`, `split` and `focus` drive the layout; `send` feeds text to
  a pane behind a scripting gate and never auto-submits; `key` sends a single
  non-submitting keystroke; `wait` blocks on pane readiness as an
  orchestration primitive.
- `paneflow up`: declarative agent workspaces. One command builds a
  workspace from a spec (per-pane cwd, agent to launch, prompt prefill),
  backed by a `workspace.up` IPC handler.
- Worktree-per-agent: a `worktree = "branch"` field in `up` gives each pane
  its own git worktree, with `.env*` copy, an optional setup command, a
  `${port_offset}` variable for port isolation, clean teardown when the
  workspace closes and pruning of orphaned worktrees at startup.
- `paneflow flow`: a flow engine for multi-agent pipelines. `flow.toml`
  declares spawn steps with `ready.pattern` barriers, gated send steps,
  `foreach` fan-out and fan-in, `capture` to pass data between steps, plus
  validation with cycle detection, `--dry-run`, reporting, exit codes and
  state resume. Submission stays double-gated end to end.
- Attention routing: a pane whose agent waits for input glows and its tab
  shows an attention dot; the desktop notification carries the agent's
  question; `Ctrl+Shift+J` cycles to the next waiting agent across
  workspaces; hovering the pane badge peeks at the question without
  stealing focus.
- Persistent agent-notification hooks: `paneflow hooks setup` installs a
  durable hook for supported agents, `paneflow hooks status` reports each
  agent honestly, and the launch shim defers to a persistent hook instead
  of injecting an ephemeral one.
- Turn-end desktop notification when the window is unfocused.

### Changed

- Agents view rebuilt as a Codex-style cockpit: rail sections (Search,
  Pinned, Projects, Chats), free chats anchored to the home directory, a
  contextual top bar with a thread overflow menu, and unified
  selection/empty states.
- Cockpit chrome across every mode: full-height rails with a floating
  rail-confined title bar, the update call-to-action moved into the sidebar
  in Cli/Agents, a single-row Diff toolbar with the scope breadcrumb
  inline, and quieter text inputs (1px white caret).
- The sessions sidebar now follows the active workspace instead of staying
  bound to the previous one.
- "PaneFlow Light" is temporarily out of the bundled theme set pending a
  light-theme redesign; configs naming it fall back to One Dark.

### Fixed

- A literal `--update-and-exit` token passed as a CLI or hooks argument can
  no longer hijack the process into the self-updater.

## [0.3.9] - 2026-06-09

- Rebuilt the native terminal engine on upstream `alacritty_terminal` with
  rendering parity: OSC 8 hyperlinks, configurable cursor shapes, a live
  scrollbar, and faithful cursor and alt-screen input handling.
- Added PTY teardown and exit-status reporting so a closed shell reports how it
  ended, plus golden snapshot tests that lock terminal rendering against
  regressions.
- Added a Terminal settings tab and a terminal configuration block in the config
  schema and loader.
- Hardened self-update end to end: release artifacts are now signed in CI, every
  download is verified against an embedded minisign key before install, updates
  swap in atomically with crash recovery, and an unsigned build refuses to
  self-update.
- Added per-platform update verification: macOS codesign and spctl gating with
  Team ID pinning, Windows Authenticode through WinVerifyTrust, hardened tar.gz
  and AppImage extraction, and native host architecture detection for Rosetta
  and WOW64.
- Eliminated panics on untrusted input across session restore, config parsing,
  IPC, date handling, and layout, replacing defensive indexing with fail-safe
  accessors.
- Bounded every external surface against resource exhaustion: the IPC server
  caps line size, concurrency, and idle time; external subprocesses run under a
  timeout with a watchdog; ingress and DoS caps are centralized in one module.
- Moved blocking work off the render thread: session saves, git diff stats,
  config loads, font enumeration, and the recursive file watcher now run in the
  background, with a cached config feeding every frame.
- Sanitized untrusted content paths: markdown rendering strips bidi and
  zero-width characters, git refs are stripped of control bytes before they
  reach agent prompts, and session ids are validated to block argument
  injection.
- Validated and clamped all persisted config and session input, with atomic
  write-and-rename for `paneflow.json` and symmetric bounds shared across
  session, IPC, and the config schema.
- Hardened terminal and shim lifecycle: PID-reuse guards, an environment
  deny-list and scrollback sanitization on session restore, codex flock
  serialization, and correct orphan cleanup under systemd.
- Improved Windows portability: portable shell launches, correct LOCALAPPDATA
  casing, Git for Windows PATH augmentation, and `dirs`-based home resolution.
- Reduced per-frame allocations in terminal paint, sidebar recompute, and
  layout, with memoized derivations and zero-allocation leaf lookups.
- Fixed non-US keyboard input, decoupled Alt-on-arrows from the option-as-meta
  setting, and reworked the keybindings editor to be action-indexed with
  collision detection.

## [0.3.8] - 2026-06-02

- Changed the Agents view to a terminal-only model: each thread now launches a
  CLI coding agent directly in a terminal pane with a pre-filled prompt instead
  of an in-app chat, keeping the agent in its native terminal with permission
  bypass respected exactly as the tab-bar buttons do.
- Added eleven launchable agents alongside Claude Code, Codex, OpenCode, Pi, and
  Hermes: Grok, Amp, Cursor, Gemini, Kiro, Antigravity, Copilot, CodeBuddy,
  Factory, Qoder, and Openclaw, each with its own tab-bar button, icon, and
  Settings visibility toggle.
- Each Terminal Thread now remembers which agent it launches and restores it on
  the next session.
- Removed the in-app ACP chat, its conversation timeline and composer, and the
  separate agent sign-in page; agents now authenticate in their own terminal.
- Hardened the Git diff viewer with safer working-tree reads, a shared
  generated-file skip-list, and a watcher-refresh race fix.
- Polished open-source onboarding: community-health files, issue templates, and
  README positioning on the agent cockpit and cross-platform story.

## [0.3.7] - 2026-06-01

- Added an in-app Git diff viewer with file trees, sticky headers, hunk jumps,
  gutter line numbers, per-file diffstats, and word-level highlighting.
- Added branch review flows that open selected agents in real terminal panes
  with a review prompt scoped to the branch worktree.
- Added hunk/file diff copy actions for sending precise context to agents.
- Improved Worktree branch-column behavior so deselecting a branch is explicit.

## [0.3.6] - 2026-05-29

- Added docked Agent Sessions and Files sidebars.
- Added markdown-file opening from the Files panel into an adjacent pane.
- Added drag-to-reorder tabs within a pane and drag-to-move tabs between panes.

## [0.3.5] - 2026-05-29

- Added the Paneflow MCP bridge so capable agents can read pane output through
  `list_panes`, `read_pane`, and `search_pane`.
- Added `paneflow mcp install`, `uninstall`, and `status` commands.
- Added readable pane references, persistent tab renames, and clipboard copy for
  pane references.

## [0.3.4] - 2026-05-28

- Hardened the CLI-agent subsystem for long sessions: bounded caches, parser
  limits, safer IPC behavior, better logging, and reduced retained UI state.
- Improved hot paths for markdown streaming, code highlighting, persisted-item
  collection, and activity-state computation.
- Added CI audit coverage and benchmark baselines for key performance paths.
- Changed `claude_code_bypass_permissions` to default to `false` on fresh
  installs.

## [0.3.3] - 2026-05-27

- Added multi-session tracking for concurrent Claude Code, Codex, and other
  agent sessions in the same workspace.
- Added Ctrl/Cmd-click handling for `file:line:column` references in terminal
  output and assistant messages.
- Added IPC singleton protection to prevent two app instances from racing over
  the same socket.
- Improved ACP client capability declarations for richer Codex and Claude Code
  streams.

## [0.3.2] - 2026-05-26

- Added Terminal Threads as first-class sidebar entries backed by Paneflow's PTY
  stack.
- Added editable project and thread names using the same text widget as the
  composer.
- Added background thread-title generation and title cleanup for agent-provided
  titles.

## [0.3.1] - 2026-05-26

- Maintenance release. See the GitHub compare link for the full commit list.

## [0.3.0] - 2026-05-25

- Opened the 0.3.x release line. See the GitHub compare link for the full commit
  list.

[Unreleased]: https://github.com/arthjean/paneflow/compare/v0.8.1...HEAD
[0.8.1]: https://github.com/arthjean/paneflow/compare/v0.8.0...v0.8.1
[0.6.2]: https://github.com/arthjean/paneflow/compare/v0.6.1...v0.6.2
[0.6.1]: https://github.com/arthjean/paneflow/compare/v0.6.0...v0.6.1
[0.6.0]: https://github.com/arthjean/paneflow/compare/v0.5.9...v0.6.0
[0.5.0]: https://github.com/arthjean/paneflow/compare/v0.4.4...v0.5.0
[0.4.4]: https://github.com/arthjean/paneflow/compare/v0.4.3...v0.4.4
[0.4.3]: https://github.com/arthjean/paneflow/compare/v0.4.2...v0.4.3
[0.4.2]: https://github.com/arthjean/paneflow/compare/v0.4.1...v0.4.2
[0.4.1]: https://github.com/arthjean/paneflow/compare/v0.4.0...v0.4.1
[0.4.0]: https://github.com/arthjean/paneflow/compare/v0.3.9...v0.4.0
[0.3.9]: https://github.com/arthjean/paneflow/compare/v0.3.8...v0.3.9
[0.3.8]: https://github.com/arthjean/paneflow/compare/v0.3.7...v0.3.8
[0.3.7]: https://github.com/arthjean/paneflow/compare/v0.3.6...v0.3.7
[0.3.6]: https://github.com/arthjean/paneflow/compare/v0.3.5...v0.3.6
[0.3.5]: https://github.com/arthjean/paneflow/releases/tag/v0.3.5
[0.3.4]: https://github.com/arthjean/paneflow/releases/tag/v0.3.4
[0.3.3]: https://github.com/arthjean/paneflow/releases/tag/v0.3.3
[0.3.2]: https://github.com/arthjean/paneflow/compare/v0.3.1...v0.3.2
[0.3.1]: https://github.com/arthjean/paneflow/compare/v0.3.0...v0.3.1
[0.3.0]: https://github.com/arthjean/paneflow/compare/v0.2.17...v0.3.0
