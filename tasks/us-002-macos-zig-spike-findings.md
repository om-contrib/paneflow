# US-002 - macOS libghostty spike findings

**PRD:** `tasks/prd-macos-libghostty-backend-2026-Q3.md` (EP-002 / US-002)
**Date:** 2026-08-15
**Status:** spike OPEN - reproducibility not yet measured, blocked before first successful build

## Summary

The pinned toolchain (Zig 0.15.2) could not produce a `libghostty-vt.a` for
`aarch64-apple-darwin` on the development machine. Two independent obstacles
were characterised; neither is caused by Paneflow code, and neither
invalidates the US-002 plan. The reproducibility question the story exists to
answer - can a Mach-O archive be normalised bit-for-bit without elfutils -
remains **unanswered**, because no archive was produced.

Two findings below are already actionable for US-003 regardless of how the
blocker is resolved.

## Environment

| Item | Value |
|---|---|
| Host | Apple Silicon, Darwin 27.0.0, host macOS detected by Zig as `27.0.0` |
| Xcode | `/Applications/Xcode.app`, newest SDK `MacOSX26.5.sdk` |
| Command Line Tools SDKs | `MacOSX15.0.sdk`, `MacOSX26.5.sdk`, `MacOSX27.0.sdk` |
| Zig | 0.15.2 `aarch64-macos`, sha256 `3cc2bab367e185cdfb27501c4b30b1b0653c28d9f73df8dc91488e66ece5fa6b`, matching the official `ziglang.org/download/index.json` entry |
| Ghostty | clean checkout of `ae52f97dcac558735cfa916ea3965f247e5c6e9e`; `include/ghostty/vt.h` matches `header_sha256` in the manifest |

Provenance inputs are therefore confirmed good. The failure is toolchain/host,
not source.

## Blocker 1 - Zig 0.15.2's Mach-O linker cannot consume the current Apple SDK on arm64

Every **native** link fails with the whole of libc undefined (`_abort`,
`_bzero`, `_getenv`, `_sigaction`, `__availability_version_check`, …).

The cause is not the OS/SDK version gap. It is the target slice list in the
SDK's text stub. Both SDKs on the machine declare:

```yaml
--- !tapi-tbd
tbd-version:     4
targets:         [ x86_64-macos, x86_64-maccatalyst, arm64e-macos, arm64e-maccatalyst ]
```

`arm64e-macos` is present, plain **`arm64-macos` is not**. Zig targets
`aarch64-macos` (= `arm64-macos`), its Mach-O linker matches the slice exactly,
finds nothing, and reports every symbol undefined. Apple's own `ld` tolerates
this; Zig 0.15.2 does not. Verified on both
`Xcode.app/.../MacOSX26.5.sdk` and `CommandLineTools/SDKs/MacOSX27.0.sdk`.

`--verbose-link` shows the two regimes clearly:

```
# native - uses the Apple SDK, fails
zig ld -dynamic -platform_version macos 27.0.0 26.5 \
  -syslibroot /Applications/Xcode.app/.../MacOSX26.5.sdk -lSystem ...

# explicit target - no -syslibroot at all, uses Zig's bundled stub, succeeds
zig ld -dynamic -platform_version macos 13.0.0 15.5 -e _main ... -lSystem
```

So an explicit target in the triple puts Zig in cross-compilation mode, where
it ignores the system SDK and uses its own
`lib/libc/darwin/libSystem.tbd` - which does carry the `arm64-macos` slice.

Measured:

| Configuration | Result |
|---|---|
| native, Xcode SDK 26.5 | fail |
| native, `SDKROOT=…/MacOSX15.0.sdk` | fail - `SDKROOT` is not honoured, `--verbose-link` shows Zig still selects 26.5 |
| native, `DEVELOPER_DIR=/Library/Developer/CommandLineTools` (SDK 27.0) | fail - same slice list |
| native, `MACOSX_DEPLOYMENT_TARGET=13.0` | fail - not honoured |
| native, `SYSTEM_VERSION_COMPAT=1` | fail - Zig still reports host 27.0.0 |
| `-target aarch64-macos` with `DEVELOPER_DIR=/nonexistent` | **links, binary runs** |
| `-target aarch64-macos.13.0.0`, SDK present | **links** - no `-syslibroot` emitted |

This fixes the artifact build but **not the build runner**: `zig build`
compiles `build.zig` itself for the native host, and `-Dtarget` does not apply
to it. `zig build` therefore fails before reaching any Ghostty step - even
`zig build --help` fails.

Note the machine runs a macOS 27 **beta**, so Xcode 27 does not exist yet and
Xcode 26.5 is the correct current release. The blocker is not a misconfigured
machine, and updating Xcode cannot resolve it today.

`xcrun --kill-cache` was required once, unrelated to this: after
`xcode-select -s`, `xcrun --show-sdk-path` kept returning a stale
CommandLineTools path.

## Blocker 2 - the pinned Ghostty build.zig requires a findable SDK

Neutralising the SDK works around blocker 1 for the build runner, but then
Ghostty's own build fails:

```
error.DarwinSdkNotFound
  std/zig/LibCInstallation.zig:174  findNative
  src/build/SharedDeps.zig:146      add
  src/build/GhosttyBench.zig:29     init
  build.zig:103                     build
```

At the pinned revision, `SharedDeps.add` calls
`std.zig.LibCInstallation.findNative()` on any Darwin target purely to obtain
`sys_include_dir` for a `translateC` of `src/os/locale.c`. Zig constructs the
entire build graph before running any step, so `GhosttyBench.init` executes
even for `-Demit-lib-vt=true`. The lib-vt-only build therefore still requires a
usable Darwin SDK.

The two blockers are mutually exclusive under the current host/SDK pair: the
build runner needs the SDK hidden, Ghostty needs it visible.

## Findings already usable for US-003

1. **The deployment target must be pinned in the triple, not inherited.**
   `-Dtarget=aarch64-macos` resolves the OS version from the build host, which
   both causes blocker 1 and would leak the builder's OS version into the
   artifact. `aarch64-macos.13.0.0` is explicit and matches the PRD's macOS 13
   floor. The manifest needs a `macos_deployment_target` key, and `build-info`
   must record it so `build.rs` can verify it - the same treatment Windows
   gives `source_date_epoch`.

2. **`ar -D` has a hermetic replacement.** Zig ships `zig ar` (llvm-ar, accepts
   `D`) and `zig objcopy`. Using them instead of the host `ar`/`strip` keeps
   the Darwin normalisation recipe as hermetic as the Linux one and removes
   Xcode from the normalisation inputs. Not yet exercised - no archive was
   produced.

## Options

| Option | Effect | Cost |
|---|---|---|
| Produce the artifact on CI (`libghostty-macos.yml`, US-012) | GitHub's macOS runners pair a stable macOS with an Xcode whose SDK still carries the `arm64-macos` slice. This is where the reviewed artifact must be produced anyway, for provenance | Reorders US-012 before US-002 closes; slower feedback while tuning the normalisation recipe |
| Wait for the machine to leave the macOS 27 beta, then re-test | The slice list may return, or a newer Xcode may ship one Zig 0.15.2 accepts | Unknown date, and unverified that it changes anything |
| Bump the pinned Zig | A newer Zig may match `arm64e` slices or fall back to its bundled stub | Invalidates every hash on all three platforms and is an explicit PRD Non-Goal |

Recommended: **CI**. An Xcode upgrade is not an option - the machine runs a
macOS 27 beta and Xcode 27 does not exist. And the local blocker is the slice
list, which is identical in both SDKs already installed, so a newer Xcode is
not known to help.

Nothing in EP-003 through EP-006 depends on producing this artifact locally
except the stories that link it. US-001 is already done, and the host-port
design work can proceed against the Linux and Windows implementations.

## Reproduction

```sh
# Zig 0.15.2, verified against ziglang.org/download/index.json
curl -fsSLO https://ziglang.org/download/0.15.2/zig-aarch64-macos-0.15.2.tar.xz
tar xf zig-aarch64-macos-0.15.2.tar.xz

git clone --filter=blob:none --no-checkout https://github.com/ghostty-org/ghostty.git
git -C ghostty checkout ae52f97dcac558735cfa916ea3965f247e5c6e9e

# Minimal reproducer for blocker 1 - fails with all of libc undefined
printf 'pub fn main() void {}\n' > probe.zig
./zig-aarch64-macos-0.15.2/zig build-exe probe.zig -lc --verbose-link

# Blocker 2
cd ghostty && DEVELOPER_DIR=/nonexistent ../zig-aarch64-macos-0.15.2/zig build \
  -Demit-lib-vt=true -Dtarget=aarch64-macos.13.0.0 -Doptimize=ReleaseFast --prefix ../out
```
