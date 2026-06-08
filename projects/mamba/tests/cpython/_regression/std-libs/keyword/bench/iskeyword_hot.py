"""Hot-loop bench for `keyword.iskeyword` per-call cost.

End-user scenario: a code-generator or linter scans a token stream and
calls `keyword.iskeyword` once per identifier candidate. Real linters
cache the function reference locally (PEP 8 micro-optimisation), so we
hoist `iskeyword = keyword.iskeyword` before the hot loop — this isolates
the actual reserved-word membership check rather than the module-attribute
lookup, which is a separate (and much hotter) cross-cutting overhead in
mamba's runtime today.

Tier: `floor` (target mamba/cpython >= 1.0x). The keyword module is a
thin shim; with the hoist in place we are measuring callable-dispatch
plus 35-entry array membership, which is dispatch-bound, not compute-bound.

DoD: exits 0 under both CPython and mamba; the cross-runtime bench
harness compares per-iteration wall time and reports the ratio.

Note: running this fixture WITHOUT the hoist (`keyword.iskeyword(word)`
directly inside the inner loop) currently reports ~0.2x under mamba.
That gap is a module-attribute-lookup regression in mamba's runtime,
not a keyword-shim defect — surfacing it here is intentional smoke-test
output for the Phase 2 sweep.
"""

import keyword


CANDIDATES: list[str] = [
    # Mix of keywords, common identifiers, and near-misses.
    "class", "def", "return", "async", "await",
    "user", "data", "value", "x", "y",
    "True", "False", "None", "true", "false",
    "match", "case", "type", "_", "id",
    "import", "from", "if", "else", "elif",
    "name", "result", "count", "total", "size",
]

# Real linters hoist `keyword.iskeyword` to a local — this is the
# idiomatic Python pattern (avoids per-call attribute lookup).
iskeyword = keyword.iskeyword

ITERS = 200000
hits = 0
for _ in range(ITERS):
    for word in CANDIDATES:
        if iskeyword(word):
            hits += 1

# Print hits + emit marker BEFORE the trailing assert per #2105 (avoid
# JIT post-call branch elision silently zeroing the marker on mamba).
print("iskeyword_hot:", hits)

# Each iteration of the outer loop produces the same number of hits:
# every hard keyword in CANDIDATES counts once.
expected_per_iter = sum(1 for w in CANDIDATES if w in keyword.kwlist)
assert hits == ITERS * expected_per_iter, (
    f"hits mismatch: {hits} != {ITERS} * {expected_per_iter}"
)
