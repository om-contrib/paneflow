//! macOS libghostty EP-001 US-001 / NFR-013.
//!
//! Backend availability is expressed by the single `paneflow_ghostty` cfg
//! emitted from `src-app/build.rs`. This test keeps it that way: it fails as
//! soon as a `cfg` expression re-derives the platform/feature disjunction by
//! naming more than one `libghostty-*` feature at once.
//!
//! Single-platform gates (one feature only) are deliberately allowed - they
//! express an OS primitive rather than backend availability, and are revisited
//! when the POSIX host is widened to Darwin (EP-003 US-005).
//!
//! A gate that legitimately needs both feature names - currently only the
//! alias/feature divergence guard in `terminal/mod.rs` - opts out with a
//! `cfg-policy-allow:` comment on a preceding line, stating why.

#![allow(
    clippy::panic,
    reason = "integration test setup failures need contextual diagnostics"
)]

use std::path::{Path, PathBuf};

const FEATURES: [&str; 2] = ["libghostty-linux", "libghostty-windows"];

/// Collect every `.rs` file below `root`.
fn rust_sources(root: &Path) -> Vec<PathBuf> {
    let mut found = Vec::new();
    let mut stack = vec![root.to_path_buf()];
    while let Some(dir) = stack.pop() {
        let entries = std::fs::read_dir(&dir)
            .unwrap_or_else(|error| panic!("cannot read {}: {error}", dir.display()));
        for entry in entries {
            let entry =
                entry.unwrap_or_else(|error| panic!("cannot read {}: {error}", dir.display()));
            let path = entry.path();
            if path.is_dir() {
                stack.push(path);
            } else if path.extension().is_some_and(|ext| ext == "rs") {
                found.push(path);
            }
        }
    }
    found
}

/// Opt-out marker for a gate that legitimately names several features.
const ALLOW_MARKER: &str = "cfg-policy-allow:";

/// How far back to look for [`ALLOW_MARKER`]: enough to cover a short comment
/// block immediately above the attribute, not enough to catch an unrelated one.
const ALLOW_LOOKBEHIND: usize = 400;

/// Yield the inner text of every `cfg(...)` / `cfg!(...)` in `text`, matching
/// parentheses so nested `all(...)` / `any(...)` are captured whole.
///
/// Bodies preceded by [`ALLOW_MARKER`] are omitted.
fn cfg_bodies(text: &str) -> Vec<&str> {
    let bytes = text.as_bytes();
    let mut bodies = Vec::new();
    let mut search_from = 0usize;
    while let Some(offset) = text[search_from..].find("cfg") {
        let start = search_from + offset;
        search_from = start + 3;

        // Reject identifiers that merely end in "cfg" (e.g. `my_cfg(`).
        if start > 0 && (bytes[start - 1].is_ascii_alphanumeric() || bytes[start - 1] == b'_') {
            continue;
        }
        // Accept both the attribute form `cfg(` and the macro form `cfg!(`.
        let mut open = search_from;
        if bytes.get(open) == Some(&b'!') {
            open += 1;
        }
        if bytes.get(open) != Some(&b'(') {
            continue;
        }

        let mut depth = 0usize;
        let mut cursor = open;
        while cursor < bytes.len() {
            match bytes[cursor] {
                b'(' => depth += 1,
                b')' => {
                    depth -= 1;
                    if depth == 0 {
                        break;
                    }
                }
                _ => {}
            }
            cursor += 1;
        }
        if depth == 0 && cursor < bytes.len() {
            let lookbehind = &text[start.saturating_sub(ALLOW_LOOKBEHIND)..start];
            if !lookbehind.contains(ALLOW_MARKER) {
                bodies.push(&text[open + 1..cursor]);
            }
            search_from = cursor + 1;
        }
    }
    bodies
}

#[test]
fn no_cfg_re_derives_ghostty_backend_availability() {
    let src = Path::new(env!("CARGO_MANIFEST_DIR")).join("src");
    let mut offenders = Vec::new();

    for path in rust_sources(&src) {
        let text = std::fs::read_to_string(&path)
            .unwrap_or_else(|error| panic!("cannot read {}: {error}", path.display()));
        if !FEATURES.iter().any(|feature| text.contains(feature)) {
            continue;
        }
        for body in cfg_bodies(&text) {
            let named = FEATURES
                .iter()
                .filter(|feature| body.contains(**feature))
                .count();
            if named > 1 {
                offenders.push(format!(
                    "{}: cfg({})",
                    path.display(),
                    body.split_whitespace().collect::<Vec<_>>().join(" ")
                ));
            }
        }
    }

    assert!(
        offenders.is_empty(),
        "these cfg expressions re-derive Ghostty backend availability instead of using \
         `paneflow_ghostty` (emitted by src-app/build.rs). Adding a platform must stay a \
         one-place change:\n  {}",
        offenders.join("\n  ")
    );
}

#[test]
fn the_ghostty_alias_is_actually_used() {
    let src = Path::new(env!("CARGO_MANIFEST_DIR")).join("src");
    let uses = rust_sources(&src)
        .iter()
        .filter(|path| {
            std::fs::read_to_string(path)
                .map(|text| text.contains("paneflow_ghostty"))
                .unwrap_or(false)
        })
        .count();

    // Guards against the alias being silently dropped from build.rs and the
    // gates reverting to per-platform copies without anyone noticing.
    assert!(
        uses > 0,
        "no source file references `paneflow_ghostty`; the backend availability alias \
         has been removed or renamed without updating this policy"
    );
}
