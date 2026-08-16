# US-002 - macOS libghostty spike findings

**PRD:** `tasks/prd-macos-libghostty-backend-2026-Q3.md` (EP-002 / US-002)
**Date:** 2026-08-15, updated 2026-08-16
**Status:** spike ANSWERED - the archive is reproducible; the reviewed hash still has to come from CI

## Summary

**The Mach-O archive is reproducible bit-for-bit.** Two builds from two clean
Zig caches produced byte-identical archive, header, bindings and build-info
under the recipe `apple-strip-S+zig-ar-D`. That was the go/no-go for the whole
macOS project, and it is green.

Getting there required identifying why the pinned Zig 0.15.2 cannot build
natively on a macOS 26+ SDK, and working around it locally. The root cause is
narrow and is documented below, because it constrains which CI runner image
this lane may use.

The hash produced locally is **not** the reviewed hash: it must be reproduced
on a clean CI runner before being written into the manifest, since `strip`
comes from the host Xcode and its version is part of the recipe.

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

## Result - the reproducible recipe

`scripts/build-libghostty-macos.sh --verify-reproducible` passes. Recorded
`build-info.txt` from the local run:

```
source_sha=ae52f97dcac558735cfa916ea3965f247e5c6e9e
zig_version=0.15.2
rust_target=aarch64-apple-darwin
zig_target=aarch64-macos.13.0.0
optimize=ReleaseFast
archive_normalization=apple-strip-S+zig-ar-D
macos_deployment_target=13.0.0
archive_sha256=31fcdfc6c8791baa45bfe4302dfd9b38a0019b9b8a0d5260f53102beea9b9c61
```

Archive: `arm64`, 1 816 304 bytes, 820 exported symbols, down from 7.5 MB
before stripping.

Three things the recipe had to get right, each found by running it rather than
by reasoning about it:

1. **`zig objcopy` cannot be used.** It only handles ELF and fails on Mach-O
   with `invalid elf file: InvalidElfMagic`. Apple's `strip -S` does the job.
   Linux also strips with a host tool (`eu-strip`), so this is not a loss of
   hermeticity relative to the existing recipe.
2. **Stripping is required, not cosmetic.** Zig bakes absolute cache paths into
   the objects: 3 occurrences in `vt.o`, 5 in `libghostty-vt-static_zcu.o`.
   Without the strip, two builds from different cache directories cannot match.
   `strip -S` removes all of them.
3. **Archive members are stored with mode `000`.** `ar x` restores them
   unreadable, so every later step fails with `AccessDenied` until the script
   restores owner read/write after extraction.

`lipo -archs` is the architecture check, not `file`: on a static library `file`
only ever reports `current ar archive`.

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

The `arm64-macos` slice is present in `MacOSX15.0.sdk` and absent from both
`MacOSX26.5.sdk` and `MacOSX27.0.sdk`, so Apple dropped it in the macOS 26 SDK.
Two consequences:

- **CI is fine.** A macOS 15 runner image (Xcode 16.x, SDK 15.x) still exports
  the slice. `libghostty-macos.yml` asserts this explicitly before building, so
  a future runner-image bump fails loudly instead of mysteriously.
- **Local builds need a shim** on any machine whose newest SDK is 26+.

## Local workaround for a macOS 26+ / beta machine

Zig resolves the SDK through `xcrun --show-sdk-path`. Putting an `xcrun` ahead
of it on `PATH` that answers with an SDK 15 path makes the native build runner
link again, with no change to system state and no `sudo`:

```sh
mkdir -p /tmp/zig-sdk-shim
cat > /tmp/zig-sdk-shim/xcrun <<'EOF'
#!/bin/sh
SDK=/Library/Developer/CommandLineTools/SDKs/MacOSX15.0.sdk
for arg in "$@"; do
  case "$arg" in
    --show-sdk-path) echo "$SDK"; exit 0 ;;
    --show-sdk-version) echo "15.0"; exit 0 ;;
  esac
done
exec /usr/bin/xcrun "$@"
EOF
chmod +x /tmp/zig-sdk-shim/xcrun

PATH="/tmp/zig-sdk-shim:$PATH" \
PANEFLOW_GHOSTTY_SOURCE_DIR=/path/to/ghostty \
  scripts/build-libghostty-macos.sh --verify-reproducible
```

This is a developer convenience only. It must never appear in the repo recipe
or in CI: the reviewed artifact has to be built against a real, declared SDK.

## Options for the reviewed hash

| Option | Effect | Cost |
|---|---|---|
| Take the hash from a CI run of `libghostty-macos.yml` | The reviewed artifact is produced where its provenance is verifiable, on a declared runner image and Xcode | One CI run before US-003 can record the manifest key |
| Take the hash from a local shimmed build | Immediate | Rejected: `strip` comes from the developer's Xcode, and the SDK is being faked. Not reviewable provenance |
| Bump the pinned Zig | A newer Zig may match `arm64e` slices | Invalidates every hash on all three platforms and is an explicit PRD Non-Goal |

Recommended: run `libghostty-macos.yml` once, read `archive_sha256` from the
run summary, and record it as `archive_sha256_aarch64_apple_darwin` in US-003.
If the CI hash differs from the local one, that difference is itself worth
understanding before the key is written - the artifact build is a
cross-compilation that should not depend on the host SDK, so only the `strip`
version should be able to move it.

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
