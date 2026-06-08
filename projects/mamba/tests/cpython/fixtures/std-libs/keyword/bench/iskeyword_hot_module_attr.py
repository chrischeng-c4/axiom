"""Regression bench for #2097 — module-attribute lookup in hot loops.

Phase 2 Task #10 surfaced a cross-cutting perf gap: idiomatic Python
code that fetches a module attribute inside a hot loop
(`keyword.iskeyword(w)` per iter) ran ~5x slower than the hoisted
form (`f = keyword.iskeyword; f(w)`). The root cause: every
`mb_getattr` / `mb_module_getattr` call cloned the attribute name
into a fresh `String` and walked the full dunder cascade before
reaching the module dict.

This fixture is the **un-hoisted** counterpart to `iskeyword_hot.py`
— it deliberately exercises `module.attr()` inside the inner loop
so the cross-runtime bench harness reports the regression directly.

Acceptance (from issue body): under mamba this should run >= 1.0x
CPython without changing the fixture. The fix lands a borrowing
fast path at the top of `mb_getattr` / `mb_module_getattr` that:

  1. Recognises a Dict receiver + Str attr (the JIT bakes attr
     names as immortal `ObjData::Str`, so the pointer is stable
     across iterations).
  2. Looks up the attr via `IndexMap::get(&str)` using the
     `Equivalent<DictKey> for str` impl — no allocation, no
     dunder cascade, no extra borrow.

Tier: `floor` (mamba/cpython >= 1.0x). Same callable, same body,
same CANDIDATES list as the hoisted bench — the only difference
is the call-site form.
"""

import keyword


CANDIDATES: list[str] = [
    "class", "def", "return", "async", "await",
    "user", "data", "value", "x", "y",
    "True", "False", "None", "true", "false",
    "match", "case", "type", "_", "id",
    "import", "from", "if", "else", "elif",
    "name", "result", "count", "total", "size",
]

# DELIBERATELY NOT HOISTED — this is the regression-shape.
# Real-world idiomatic Python (`mod.func()` inside a loop) lives here.
ITERS = 200000
hits = 0
for _ in range(ITERS):
    for word in CANDIDATES:
        if keyword.iskeyword(word):
            hits += 1

# Print hits + emit marker BEFORE the trailing assert per #2105 (avoid
# JIT post-call branch elision silently zeroing the marker on mamba).
print("iskeyword_hot_module_attr:", hits)

expected_per_iter = sum(1 for w in CANDIDATES if w in keyword.kwlist)
assert hits == ITERS * expected_per_iter, (
    f"hits mismatch: {hits} != {ITERS} * {expected_per_iter}"
)
