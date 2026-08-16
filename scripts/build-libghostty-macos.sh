#!/usr/bin/env bash
# Prepare the reviewed libghostty-vt archive for aarch64-apple-darwin.
#
# Mirrors scripts/build-libghostty-linux.sh. Two Darwin-specific decisions,
# both forced by findings in tasks/us-002-macos-zig-spike-findings.md:
#
#   1. The macOS deployment target is pinned in the Zig triple. Leaving it
#      implicit resolves it from the build host, which leaks the builder's OS
#      version into the artifact and, when the host is newer than its SDK,
#      fails to link at all. Pinning it also keeps Zig in cross-compilation
#      mode, where it uses its bundled libSystem stub rather than the Apple
#      SDK - the SDK's text stub lists arm64e-macos but not arm64-macos, which
#      Zig's Mach-O linker requires.
#
#   2. Normalization is `strip -S` plus `zig ar crsD`. Darwin has no elfutils
#      and BSD `ar` has no deterministic `D` mode, so `zig ar` (llvm-ar)
#      supplies it. `zig objcopy` is not an option: it only handles ELF.
#      Stripping is required, not cosmetic - Zig bakes absolute cache paths
#      into the objects' debug data.
#
# Deliberately avoids bash 4 builtins (`mapfile`): GitHub's macOS runners ship
# bash 3.2.
set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
MANIFEST="$ROOT/native/libghostty/manifest.toml"
SOURCE_DIR="${PANEFLOW_GHOSTTY_SOURCE_DIR:-}"
VERIFY_REPRODUCIBLE=0
TARGETS=()

manifest_string() {
  local key="$1"
  sed -n "s/^${key} = \"\(.*\)\"$/\1/p" "$MANIFEST"
}

while (($#)); do
  case "$1" in
    --target)
      [[ $# -ge 2 ]] || { echo "--target requires a Rust target triple" >&2; exit 2; }
      TARGETS+=("$2")
      shift 2
      ;;
    --verify-reproducible)
      VERIFY_REPRODUCIBLE=1
      shift
      ;;
    *)
      echo "unknown argument: $1" >&2
      exit 2
      ;;
  esac
done

SOURCE_SHA="$(manifest_string source_sha)"
ZIG_VERSION="$(manifest_string zig_version)"
HEADER_PATH="$(manifest_string header_path)"
HEADER_SHA256="$(manifest_string header_sha256)"
BINDINGS_PATH="$(manifest_string bindings_path)"
BINDINGS_SHA256="$(manifest_string bindings_sha256)"
BUILD_MODE="$(manifest_string build_mode)"
ARCHIVE_PATH="$(manifest_string archive_path)"
ARCHIVE_NORMALIZATION="$(manifest_string archive_normalization_macos)"
DEPLOYMENT_TARGET="$(manifest_string macos_deployment_target)"
BUILD_INFO_SYMBOL="$(manifest_string build_info_symbol)"

[[ -n "$SOURCE_DIR" ]] || {
  echo "PANEFLOW_GHOSTTY_SOURCE_DIR must point to Ghostty $SOURCE_SHA" >&2
  exit 1
}
git -C "$SOURCE_DIR" rev-parse --is-inside-work-tree >/dev/null 2>&1 || {
  echo "$SOURCE_DIR is not a Ghostty Git checkout" >&2
  exit 1
}
ACTUAL_SHA="$(git -C "$SOURCE_DIR" rev-parse HEAD)"
[[ "$ACTUAL_SHA" == "$SOURCE_SHA" ]] || {
  echo "Ghostty source mismatch: expected $SOURCE_SHA, got $ACTUAL_SHA" >&2
  exit 1
}
SOURCE_STATUS="$(git -C "$SOURCE_DIR" status --porcelain --untracked-files=all)"
[[ -z "$SOURCE_STATUS" ]] || {
  echo "Ghostty source must be a clean checkout of $SOURCE_SHA" >&2
  printf '%s\n' "$SOURCE_STATUS" >&2
  exit 1
}
command -v zig >/dev/null || {
  echo "libghostty requires Zig $ZIG_VERSION; install or select the pinned toolchain" >&2
  exit 1
}
ACTUAL_ZIG="$(zig version)"
[[ "$ACTUAL_ZIG" == "$ZIG_VERSION" ]] || {
  echo "libghostty requires Zig $ZIG_VERSION, found $ACTUAL_ZIG" >&2
  exit 1
}
command -v shasum >/dev/null || { echo "shasum is required" >&2; exit 1; }
command -v nm >/dev/null || { echo "nm is required to verify exported symbols" >&2; exit 1; }
command -v strip >/dev/null || { echo "strip is required to normalize release archives" >&2; exit 1; }
command -v lipo >/dev/null || { echo "lipo is required to verify the archive architecture" >&2; exit 1; }

sha256_of() {
  shasum -a 256 "$1" | awk '{print $1}'
}

ACTUAL_HEADER_SHA256="$(sha256_of "$SOURCE_DIR/$HEADER_PATH")"
[[ "$ACTUAL_HEADER_SHA256" == "$HEADER_SHA256" ]] || {
  echo "Ghostty header checksum mismatch: expected $HEADER_SHA256, got $ACTUAL_HEADER_SHA256" >&2
  exit 1
}
ACTUAL_BINDINGS_SHA256="$(sha256_of "$ROOT/$BINDINGS_PATH")"
[[ "$ACTUAL_BINDINGS_SHA256" == "$BINDINGS_SHA256" ]] || {
  echo "Paneflow bindings checksum mismatch: expected $BINDINGS_SHA256, got $ACTUAL_BINDINGS_SHA256" >&2
  exit 1
}

if ((${#TARGETS[@]} == 0)); then
  TARGETS=("aarch64-apple-darwin")
fi

zig_target() {
  case "$1" in
    aarch64-apple-darwin) echo "aarch64-macos.$DEPLOYMENT_TARGET" ;;
    *) echo "unsupported macOS target: $1" >&2; return 1 ;;
  esac
}

normalize_archive() {
  local archive="$1"
  local normalize_dir="$archive.normalize"
  local normalized="$archive.normalized"
  local member
  local duplicates
  local members_file="$archive.members"
  local sorted_file="$archive.members.sorted"

  zig ar t "$archive" > "$members_file"
  : > "$sorted_file"
  while IFS= read -r member; do
    [[ -n "$member" ]] || continue
    printf '%s\n' "${member##*/}" >> "$sorted_file"
  done < "$members_file"
  duplicates="$(LC_ALL=C sort "$sorted_file" | uniq -d)"
  [[ -z "$duplicates" ]] || {
    echo "archive normalization found duplicate member names: $duplicates" >&2
    rm -f "$members_file" "$sorted_file"
    return 1
  }
  # Zig may append members in parallel completion order. Rebuild from a
  # canonical order so identical object files always produce identical bytes.
  LC_ALL=C sort "$sorted_file" -o "$sorted_file"

  rm -rf "$normalize_dir" "$normalized"
  mkdir -p "$normalize_dir"
  (
    cd "$normalize_dir" || exit 1
    zig ar x "$archive" || exit 1
    # Members are stored with mode 000, so `ar x` restores them unreadable and
    # every later step fails with AccessDenied. Restore owner read/write before
    # touching them.
    chmod u+rw ./*.o || exit 1
    # Zig bakes absolute cache paths into the objects' debug data. Dropping it
    # is what makes two builds from different cache dirs comparable; measured,
    # not assumed. `zig objcopy` is not usable here - it only handles ELF.
    while IFS= read -r member; do
      [[ -f "$member" ]] || exit 1
      strip -S "$member" || exit 1
    done < "$sorted_file"
    # `D` is llvm-ar's deterministic mode: zero mtime, uid, gid and mode.
    xargs zig ar crsD "$normalized" < "$sorted_file" || exit 1
  ) || {
    local status=$?
    rm -rf "$normalize_dir" "$normalized"
    rm -f "$members_file" "$sorted_file"
    return "$status"
  }
  mv "$normalized" "$archive"
  rm -rf "$normalize_dir"
  rm -f "$members_file" "$sorted_file"
}

build_one() {
  local rust_target="$1"
  local output="$2"
  local cache="$3"
  local target
  target="$(zig_target "$rust_target")"
  rm -rf "$output" "$cache"
  mkdir -p "$output" "$cache"
  (
    cd "$SOURCE_DIR"
    ZIG_GLOBAL_CACHE_DIR="$cache/global" ZIG_LOCAL_CACHE_DIR="$cache/local" zig build \
      -Demit-lib-vt=true \
      -Dtarget="$target" \
      -Doptimize="$BUILD_MODE" \
      --prefix "$output"
  )
  local archive="$output/$ARCHIVE_PATH"
  [[ -f "$archive" ]] || { echo "missing static archive: $archive" >&2; return 1; }
  [[ -f "$output/$HEADER_PATH" ]] || { echo "missing installed header: $output/$HEADER_PATH" >&2; return 1; }
  case "$ARCHIVE_NORMALIZATION" in
    apple-strip-S+zig-ar-D) normalize_archive "$archive" ;;
    *) echo "unsupported archive normalization: $ARCHIVE_NORMALIZATION" >&2; return 1 ;;
  esac
  # Mach-O prefixes C symbols with an underscore, unlike ELF.
  nm -g "$archive" | grep -E "[[:space:]][A-TV-Z][[:space:]]_${BUILD_INFO_SYMBOL}$" >/dev/null || {
    echo "archive does not export $BUILD_INFO_SYMBOL: $archive" >&2
    return 1
  }
  # An artifact carrying the wrong architecture must never reach the manifest.
  # `file` only reports "current ar archive" for a static library, so read the
  # slice list instead - `lipo -archs` prints exactly `arm64` here.
  local archive_archs
  archive_archs="$(lipo -archs "$archive")"
  [[ "$archive_archs" == "arm64" ]] || {
    echo "archive architecture is '$archive_archs', expected 'arm64': $archive" >&2
    return 1
  }
  local archive_sha
  archive_sha="$(sha256_of "$archive")"
  cp "$ROOT/$BINDINGS_PATH" "$output/bindings.rs"
  {
    echo "source_sha=$SOURCE_SHA"
    echo "zig_version=$ZIG_VERSION"
    echo "header_sha256=$HEADER_SHA256"
    echo "bindings_sha256=$BINDINGS_SHA256"
    echo "rust_target=$rust_target"
    echo "zig_target=$target"
    echo "optimize=$BUILD_MODE"
    echo "archive_normalization=$ARCHIVE_NORMALIZATION"
    echo "macos_deployment_target=$DEPLOYMENT_TARGET"
    echo "archive_sha256=$archive_sha"
    echo "build_info_symbol=$BUILD_INFO_SYMBOL"
  } > "$output/build-info.txt"
}

for rust_target in "${TARGETS[@]}"; do
  output="$ROOT/target/libghostty/$rust_target"
  cache="$ROOT/target/libghostty-cache/$rust_target"
  build_one "$rust_target" "$output" "$cache"

  if ((VERIFY_REPRODUCIBLE)); then
    second_output="$(mktemp -d)"
    second_cache="$(mktemp -d)"
    trap 'rm -rf "$second_output" "$second_cache"' EXIT
    build_one "$rust_target" "$second_output" "$second_cache"
    cmp "$output/$ARCHIVE_PATH" "$second_output/$ARCHIVE_PATH"
    cmp "$output/$HEADER_PATH" "$second_output/$HEADER_PATH"
    cmp "$output/bindings.rs" "$second_output/bindings.rs"
    cmp "$output/build-info.txt" "$second_output/build-info.txt"
    rm -rf "$second_output" "$second_cache"
    trap - EXIT
  fi

  echo "prepared $rust_target at $output"
done
