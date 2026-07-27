# agy-dispatch profile — mamba corpus mechanics

Supervisor-side. The executor never reads this; it exists so a ticket author
selects on the corpus's own structure instead of inventing a regex.

Authoritative layout doc: `projects/mamba/tests/harness/cpython/conventions/FIXTURE-LAYOUT.md`.
Read it before authoring or decomposing fixtures. This file holds only what a
**dispatch** needs.

## Two trees, and they are not interchangeable

| Tree | Contents | What it is |
|---|---|---|
| `projects/mamba/tests/cpython/` | 46,639 `.py`, of which **44,888** carry a `[tool.mamba]` PEP 723 block | the fixture corpus — the thing a denominator counts |
| `projects/mamba/tests/cpython_ported/` | 967 `.rs` | the **generated** Rust tests, one `#[test]` fn per fixture, each embedding the fixture verbatim in a raw string |

Counted 2026-07-27, `.cache` excluded. The 1,751 `.py` without a `[tool.mamba]`
block are helpers and scaffolding — they are not fixtures and must not enter a
candidate surface.

A ticket that says "tests" must say **which tree**. A count over the `.rs` tree
and a count over the `.py` tree answer different questions, and only the `.py`
tree carries the structured key below.

## The structured key — select on this first

Every fixture opens with a PEP 723 block. All 44,888 carry the same twelve keys:

    # /// script
    # requires-python = ">=3.12"
    # dependencies = []
    #
    # [tool.mamba]
    # bucket = "std-libs"
    # lib = "enum"
    # dimension = "behavior"
    # case = "intenum_compares_and_arithmetic"
    # subject = "enum.IntEnum"
    # kind = "semantic"
    # xfail = ""
    # mem_carveout = ""
    # source = "Lib/test/test_enum.py"
    # status = "filled"
    # ///

`xfail` is present on 42,394 and `mem_carveout` on 44,887 — treat a missing key
as empty, never as a parse failure.

`dimension` partitions the whole corpus:

| dimension | fixtures |
|---|---|
| behavior | 28,308 |
| type | 8,769 |
| surface | 5,494 |
| real_world | 1,039 |
| errors | 994 |
| security | 204 |
| perf | 74 |
| concurrency | 6 |

**Select on `lib`, `dimension`, `bucket`, `source` before any regex touches free
text.** The one trap this corpus has actually sprung: a non-delimiter-aware
pattern matched a 33-character *header* fragment instead of the body, on 12,647
of 13,767 items, and the hits it did return were merely the fixtures whose text
happened not to trip it early.

`source` is the hard-floor field. When a work root's Promise names a CPython
suite, every fixture whose `source` is that `Lib/test/test_*.py` is in the floor
by construction — derive it from the corpus, never from the report.

## The witness

Two fixture shapes coexist, and the witness differs:

- **plain** — PEP 723 header, docstring, executed body. Witness = everything
  below the closing `# ///`.
- **unittest-translated** (14,511 fixtures, `test_case__*.py`) — header, an
  imported CPython prologue, `# --- test body ---`, then the executed tail.
  Witness = everything below the marker.

The prologue is the false-positive factory: it is imported scaffolding the
fixture never runs, and it can manufacture a `SyntaxError` token, an
`assertRaises`, or an import of any module by the dozen. #2640 admitted
`test_consts_in_conditionals` on exactly that — a token living only in a
prologue helper the executed tail never calls.

## Building the index

One pass over 44,888 files takes seconds and serves every later oracle:

    corpus_index.json  = [{path, lib, dimension, bucket, case, subject, kind,
                           xfail, source, status, witness}]

Both the false-positive scan (admitted items whose witness never mentions the
target behaviour) and the false-negative scan (whole-corpus structural search
for items absent from the result) run off it. The false-negative scan must run
over the index, never over the executor's shortlist — the shortlist is the
executor's construct and reusing it inherits its blind spot.

## Fabrication tells specific to this corpus

- A denominator for a work root whose Promise names a CPython suite, that omits
  fixtures whose `source` is that suite. Impossible if the corpus was read.
- A count of `.py` fixtures that exceeds 44,888, or of `.rs` tests that exceeds
  the `--list` output of the pinned binary.
- A "per-item audit" citing fixture paths under `tests/cpython_ported/` — that
  tree has no `.py` files at all.
- Group size equal to shortlist size within any one `lib` — the predicate never
  ran there.

## The pinned binary

`cpython_ported_integration` is the harness for the `.rs` tree.
`<BIN> --list <filter>` enumerates; `<BIN> --test-threads=1 <filter>` runs.
Module paths use underscores where the directory uses hyphens:
`tests/cpython_ported/gen/behavior/std-libs/ast.rs` →
`cpython_ported::gen::behavior::std_libs::ast::`. A ticket that pastes the
directory path as a filter selects nothing and reports a healthy zero.
