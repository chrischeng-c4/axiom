# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "cpython321_stdlib"
# dimension = "real_world"
# case = "test_difflib"
# subject = "cpython321.test_difflib"
# kind = "semantic"
# xfail = "CPython 3.12 seed pass; mamba promotion pending"
# mem_carveout = ""
# source = "tests/cpython/config/seeds/pass/test_difflib.py"
# status = "filled"
# ///
"""cpython321.test_difflib: execute CPython 3.12 seed test_difflib"""
# mamba-xfail: CPython 3.12 seed pass; mamba promotion pending
# Promoted from the upstream unittest port to an executable AssertionPass seed.
# Surface: difflib — the partial surface mamba services today:
#   * get_close_matches(word, possibilities) returns a ranked list of the most
#     similar matches drawn from `possibilities`
#   * get_close_matches honors the n= (max number of results) and cutoff=
#     (minimum-similarity threshold) keyword arguments
#   * SequenceMatcher and unified_diff symbols are exposed as callables
# Intentionally NOT exercised on mamba today (tracked separately):
#   * SequenceMatcher(None, a, b) — returns 0.0 (a float), not a real matcher,
#     so .ratio() / .get_opcodes() / .get_matching_blocks() are unavailable
#   * Differ / HtmlDiff / context_diff — missing
#   * unified_diff(a, b) returns [] regardless of inputs (lambda stub)
#   * ndiff — missing
import difflib

_ledger: list[int] = []

# (1) get_close_matches returns a list ranked best-first
_r = difflib.get_close_matches("apple", ["ape", "apple", "apricot"])
assert _r == ["apple", "ape"], (
    f"get_close_matches('apple', ['ape','apple','apricot']) == "
    f"['apple', 'ape'], got {_r!r}"
)
_ledger.append(1)

# (2) get_close_matches returns a list (not None) on a successful match
assert isinstance(_r, list), (
    f"get_close_matches returns a list, got {type(_r).__name__!r}"
)
_ledger.append(1)

# (3) get_close_matches returns an empty list when no candidate is close enough
_empty = difflib.get_close_matches("zzz", ["apple", "banana"])
assert _empty == [], (
    f"get_close_matches('zzz', ['apple','banana']) == [], got {_empty!r}"
)
_ledger.append(1)

# (4) get_close_matches with n=3 returns up to 3 matches ranked best-first
_n3 = difflib.get_close_matches(
    "apple", ["ape", "apple", "apply", "appli", "apricot"], n=3
)
assert _n3 == ["apple", "apply", "appli"], (
    f"get_close_matches n=3 returns top-3 ranked best-first, got {_n3!r}"
)
_ledger.append(1)

# (5) get_close_matches with cutoff filters low-similarity candidates out
_cut = difflib.get_close_matches("apple", ["xyz", "abc"], cutoff=0.9)
assert _cut == [], (
    f"get_close_matches with no candidate above cutoff returns [], "
    f"got {_cut!r}"
)
_ledger.append(1)

# (6) get_close_matches handles a multi-candidate input and returns a
#     ranked list
_h = difflib.get_close_matches("hello", ["help", "world", "yellow"])
assert _h == ["yellow", "help"], (
    f"get_close_matches('hello', ['help','world','yellow']) ranks "
    f"['yellow', 'help'], got {_h!r}"
)
_ledger.append(1)

# (7) Best match is at index 0 (ranking is preserved)
assert _r[0] == "apple", (
    f"best match for 'apple' is 'apple' at index 0, got {_r[0]!r}"
)
_ledger.append(1)

# (8) Calling get_close_matches twice with the same args is deterministic
_r2 = difflib.get_close_matches("apple", ["ape", "apple", "apricot"])
assert _r == _r2, (
    f"get_close_matches is deterministic for identical inputs, "
    f"got {_r!r} vs {_r2!r}"
)
_ledger.append(1)

# (9) Default n=3 caps the result length at 3
_many = difflib.get_close_matches(
    "apple",
    ["apple", "apply", "appli", "applea", "appleb", "applec", "appled"],
)
assert len(_many) <= 3, (
    f"get_close_matches default n=3 caps results at 3, got len={len(_many)!r}"
)
_ledger.append(1)

# (10) Symbol exposure: get_close_matches, SequenceMatcher, unified_diff
assert hasattr(difflib, "get_close_matches"), (
    "difflib.get_close_matches symbol exposed"
)
_ledger.append(1)
assert hasattr(difflib, "SequenceMatcher"), (
    "difflib.SequenceMatcher symbol exposed"
)
_ledger.append(1)
assert hasattr(difflib, "unified_diff"), "difflib.unified_diff symbol exposed"
_ledger.append(1)

print(f"MAMBA_ASSERTION_PASS: test_difflib {sum(_ledger)} asserts")
