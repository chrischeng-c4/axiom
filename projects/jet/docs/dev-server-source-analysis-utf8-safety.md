# dev source analysis panics on non-ASCII TSX/i18n strings

> **Issue**: #1485
> **Crate**: `cclab-jet` (`projects/jet/src/dev_server/source_analysis.rs`)
> **Type**: bug

## Problem

The jet dev server's import scanner accumulates characters into a
`String` buffer and, when the buffer overflows past 10 bytes without
hitting an `import` keyword, trims the prefix by calling
`buf.drain(..buf.len() - 6)`. The math assumes 1 byte = 1 character —
fine for ASCII source, but a hard failure on UTF-8 source containing
CJK / accented copy.

`String::drain` takes a *byte* range, but a `RangeTo` whose end falls
inside a multi-byte UTF-8 codepoint trips an `is_char_boundary`
assertion and panics the tokio worker thread:

```text
thread tokio-rt-worker panicked at projects/jet/src/dev_server/source_analysis.rs:35:21:
assertion failed: self.is_char_boundary(end)
```

Reproducer (from the issue):

1. `npm run dev` (maps to `jet dev -p 3212`)
2. Open a route under the Traditional Chinese i18n surface
3. `extract_imports_from_source` is called over TSX with zh-TW string
   literals — the buffer overflow trimmer runs against a buffer that
   ends mid-codepoint, the drain assertion fires.

The dev request itself still serves (the scan runs in a separate
task), but every i18n-bearing source file logs a panic, polluting
the console and masking real errors.

## Scope

In:

- Fix the prefix-trim to honour UTF-8 codepoint boundaries. The
  intent is "keep at most ~6 trailing bytes so we can still detect
  the `import` keyword growing onto the end" — a memory cap, not a
  correctness mechanism. Trimming to the nearest char boundary at
  or after `buf.len() - 6` keeps the cap (still ≤ 6 bytes retained)
  and avoids the panic. Losing a few extra trailing bytes from a
  multi-byte codepoint cannot cause a false negative: the next
  iteration's `chars().next()` push grows the buffer again, and
  CJK/accented characters cannot prefix the ASCII keyword `import`
  in any way that the trim could affect.
- Add regression tests that:
  - Run `extract_imports_from_source` over UTF-8 source containing
    CJK / accented copy and confirm no panic + correct imports
    returned.
  - Pin the boundary-safe trim helper as a unit test so future
    edits cannot regress it.

Out:

- Rewriting the import scanner. The byte-trimmer is the only known
  UTF-8 hazard in the file; a broader refactor (e.g. swap to a
  proper tokenizer) is a separate slice.
- Source-map / HMR-payload UTF-8 sweeps. This bug fix is scoped to
  the one reported panic site.

## Interface

The fix is internal to `extract_imports_from_source`. The public
function signature is unchanged:

```rust
pub fn extract_imports_from_source(code: &str) -> Vec<String>;
```

A new pure helper carries the trim invariant so it's testable in
isolation:

```rust
/// Trim the prefix of `buf` to keep the last ≤ `tail_bytes` bytes
/// AND respect UTF-8 char boundaries. Equivalent in spirit to
/// `buf.drain(..buf.len().saturating_sub(tail_bytes))` but never
/// panics on multi-byte codepoints — the split point is rounded
/// forward to the next char boundary, which can only drop *more*
/// of the prefix, never less.
fn trim_buf_to_tail_chars(buf: &mut String, tail_bytes: usize);
```

## Acceptance Criteria

- [x] `extract_imports_from_source` over UTF-8 input with CJK string
      literals does not panic.
- [x] The trim helper rounds the split point forward to the nearest
      char boundary; remaining tail is at most `tail_bytes` bytes,
      always ends with a complete codepoint.
- [x] Existing ASCII-only behaviour preserved — the prior regression
      tests still pass byte-for-byte.
- [x] New regression test covers a TSX-like snippet with a
      Traditional Chinese string literal and a subsequent relative
      `import` to assert both no-panic AND correct import
      extraction.
- [x] `cargo test -p cclab-jet --lib dev_server::source_analysis`
      passes.

## Reference Context

- `projects/jet/src/dev_server/source_analysis.rs` — file under fix.
- Rust `String::drain` documents the panic: "Panics if the starting
  point or end point do not lie on a char boundary".
