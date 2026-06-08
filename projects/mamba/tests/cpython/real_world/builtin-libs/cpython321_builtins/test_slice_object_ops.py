# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "builtin-libs"
# lib = "cpython321_builtins"
# dimension = "real_world"
# case = "test_slice_object_ops"
# subject = "cpython321.test_slice_object_ops"
# kind = "semantic"
# xfail = "CPython 3.12 seed pass; mamba promotion pending"
# mem_carveout = ""
# source = "tests/cpython/config/seeds/pass/test_slice_object_ops.py"
# status = "filled"
# ///
"""cpython321.test_slice_object_ops: execute CPython 3.12 seed test_slice_object_ops"""
# mamba-xfail: CPython 3.12 seed pass; mamba promotion pending
# Operational AssertionPass seed for the built-in `slice()` object
# itself — constructor arity, attribute access (`.start` / `.stop` /
# `.step`), and `.indices(length)` resolution. Existing slice seeds
# (lang_slice_notation, test_list_slice_advanced_ops) cover `lst[a:b:c]`
# at the syntax-sugar level, but skip the slice OBJECT API. mamba
# 0.3.60 supports the constructor + attribute access + `.indices()`
# paths matched here; it does NOT support applying a slice object via
# `lst[slice_obj]` indexing (that path returns None) — that gap is
# tracked separately and is NOT exercised here.
#
# Surface:
#   • slice(stop) — 1-arg form: start=None, stop=N, step=None;
#   • slice(start, stop) — 2-arg form: step=None;
#   • slice(start, stop, step) — 3-arg form: all three fields;
#   • .start / .stop / .step expose the constructor arguments verbatim
#     (no normalisation; `slice(2, 8)` keeps `.step is None`);
#   • Negative-valued start/stop are stored verbatim — no clamping at
#     construction time (that happens in `.indices(length)`);
#   • .indices(length) resolves None-and-negative fields against a
#     concrete sequence length and returns a 3-tuple of normalised
#     (start, stop, step) ints suitable for `range(*tup)`;
#   • .indices() handles all four key regimes — forward full
#     (slice(None, None, 1)), forward sliced (slice(2, 5)), strided
#     forward (slice(None, None, 2)), and full reverse
#     (slice(None, None, -1)) which normalises to (length-1, -1, -1);
#   • Out-of-range positive args are clamped to `length`
#     (slice(100, 200).indices(5) → (5, 5, 1) — empty-slice tuple);
#   • Negative `start` resolves via `length + start`
#     (slice(-3, None).indices(10) → (7, 10, 1)).
_ledger: list[int] = []

# 1-arg form: only `stop` is set
_s1 = slice(5)
assert _s1.start is None; _ledger.append(1)
assert _s1.stop == 5; _ledger.append(1)
assert _s1.step is None; _ledger.append(1)

# 2-arg form: start + stop, step stays None
_s2 = slice(2, 8)
assert _s2.start == 2; _ledger.append(1)
assert _s2.stop == 8; _ledger.append(1)
assert _s2.step is None; _ledger.append(1)

# 3-arg form: all three fields populated
_s3 = slice(1, 10, 2)
assert _s3.start == 1; _ledger.append(1)
assert _s3.stop == 10; _ledger.append(1)
assert _s3.step == 2; _ledger.append(1)

# Negative-valued args are stored verbatim (no normalisation at
# construction time — that's `.indices(length)`'s job)
_s4 = slice(-3, -1)
assert _s4.start == -3; _ledger.append(1)
assert _s4.stop == -1; _ledger.append(1)
assert _s4.step is None; _ledger.append(1)

# Negative step — `slice(None, None, -1)` is the full-reverse slice
_s5 = slice(None, None, -1)
assert _s5.start is None; _ledger.append(1)
assert _s5.stop is None; _ledger.append(1)
assert _s5.step == -1; _ledger.append(1)

# .indices(length) — forward full
assert slice(None, None, 1).indices(10) == (0, 10, 1); _ledger.append(1)

# .indices(length) — forward sliced
assert slice(2, 5).indices(10) == (2, 5, 1); _ledger.append(1)

# .indices(length) — strided forward
assert slice(None, None, 2).indices(10) == (0, 10, 2); _ledger.append(1)

# .indices(length) — full reverse normalises to (length-1, -1, -1)
assert slice(None, None, -1).indices(10) == (9, -1, -1); _ledger.append(1)

# .indices(length) — explicit 3-arg slice
assert slice(2, 5, 2).indices(20) == (2, 5, 2); _ledger.append(1)

# .indices(length) — negative start resolves via length + start
assert slice(-3, None).indices(10) == (7, 10, 1); _ledger.append(1)

# .indices(length) — negative start, smaller length
assert slice(-3, None).indices(5) == (2, 5, 1); _ledger.append(1)

# .indices(length) — out-of-range positive args clamp to length
# (slice(100, 200) over a length-5 sequence yields the empty-slice
# tuple (5, 5, 1))
assert slice(100, 200).indices(5) == (5, 5, 1); _ledger.append(1)

# .indices(length) — empty slice on empty sequence
assert slice(None, None, 1).indices(0) == (0, 0, 1); _ledger.append(1)

# .indices(length) — 1-arg slice resolves start to 0 and stop to N
assert slice(7).indices(10) == (0, 7, 1); _ledger.append(1)

# .indices(length) — stop past end clamps
assert slice(0, 100).indices(5) == (0, 5, 1); _ledger.append(1)

# .indices(length) — negative step with explicit start
assert slice(8, None, -1).indices(10) == (8, -1, -1); _ledger.append(1)

print(f"MAMBA_ASSERTION_PASS: test_slice_object_ops {sum(_ledger)} asserts")
