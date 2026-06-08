---
number: 904
title: "jet-build: DCE and minifier assume ASCII-only source — audit all char-index-as-byte-offset usage"
state: open
labels: [bug, crate:jet]
group: "jet-build-aot"
---

# #904 — jet-build: DCE and minifier assume ASCII-only source — audit all char-index-as-byte-offset usage

## Problem

`dce.rs` panicked on `✓` (3-byte UTF-8) because it used `Vec<char>` indices as byte offsets when slicing `&str`. Fixed in 5a880866 by adding a `byte_offsets` lookup table.

**However, the same pattern likely exists in other bundler files** that use `chars().collect::<Vec<char>>()` then index into `source[i..j]`:

## Files to Audit

| File | Lines | Pattern |
|------|-------|---------|
| `minify.rs` | 637 | `chars: Vec<char>` + `source[...]` slicing |
| `mangle.rs` | 665 | `chars: Vec<char>` + `source[...]` slicing |
| `fold.rs` | 706 | `chars: Vec<char>` + `source[...]` slicing |
| `tree_shake.rs` | 619 | May have similar patterns |
| `imports.rs` | 311 | Uses Tree-sitter (likely safe, but verify) |

## Fix Pattern

Apply the same fix as `dce.rs`:
```rust
fn build_byte_offsets(source: &str) -> Vec<usize> {
    let mut offsets: Vec<usize> = source.char_indices().map(|(i, _)| i).collect();
    offsets.push(source.len());
    offsets
}
```

Then replace all `source[i..j]` with `source[bo[i]..bo[j]]`.

## Test

Add multi-byte UTF-8 test cases to each file's test suite (strings with emoji, CJK, accented chars).

## References
- Fix commit: 5a880866
- `crates/cclab-jet/src/bundler/dce.rs` — reference implementation
