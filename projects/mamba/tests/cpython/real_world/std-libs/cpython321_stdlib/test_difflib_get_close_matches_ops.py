# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "cpython321_stdlib"
# dimension = "real_world"
# case = "test_difflib_get_close_matches_ops"
# subject = "cpython321.test_difflib_get_close_matches_ops"
# kind = "semantic"
# xfail = "CPython 3.12 seed pass; mamba promotion pending"
# mem_carveout = ""
# source = "tests/cpython/config/seeds/pass/test_difflib_get_close_matches_ops.py"
# status = "filled"
# ///
"""cpython321.test_difflib_get_close_matches_ops: execute CPython 3.12 seed test_difflib_get_close_matches_ops"""
# mamba-xfail: CPython 3.12 seed pass; mamba promotion pending
# Operational AssertionPass seed for `difflib.get_close_matches`,
# the fuzzy-match helper used for typo suggestions and similar
# nearest-string lookups. Surface: `get_close_matches(word,
# possibilities)` returns a list (possibly empty) of strings from
# `possibilities` that are sufficiently similar to `word`. Truly
# dissimilar candidates are excluded. The `cutoff=` parameter raises
# or lowers the similarity threshold (cutoff=0.9 excludes weak
# matches; cutoff=0.0 admits anything). The result type is always
# `list[str]`. Empty `possibilities` returns the empty list; exact
# matches are always included.
import difflib
_ledger: list[int] = []

# Standard fuzzy-match: typo-style match returns matching candidates
m = difflib.get_close_matches("apple", ["ape", "apply", "banana", "applet"])
assert isinstance(m, list); _ledger.append(1)
assert "apply" in m; _ledger.append(1)
assert "applet" in m; _ledger.append(1)
assert "banana" not in m; _ledger.append(1)

# High cutoff excludes everything not very close
m2 = difflib.get_close_matches("zzz", ["apple"], cutoff=0.9)
assert m2 == []; _ledger.append(1)

# Result elements are always strings
m4 = difflib.get_close_matches("apple", ["apple", "ape", "apply"])
assert all(isinstance(x, str) for x in m4); _ledger.append(1)

# Exact match is always included
assert "apple" in m4; _ledger.append(1)

# Empty possibilities → empty result
m6 = difflib.get_close_matches("x", []); assert m6 == []; _ledger.append(1)

# Typo-style nearest-neighbour
m7 = difflib.get_close_matches("hello", ["hallo", "hellp", "world"])
assert isinstance(m7, list); _ledger.append(1)
assert len(m7) >= 1; _ledger.append(1)

# Exact match present among candidates
m8 = difflib.get_close_matches("foo", ["foo", "bar", "baz"])
assert "foo" in m8; _ledger.append(1)

# Very dissimilar string with high cutoff → empty
m9 = difflib.get_close_matches("abcdefg", ["xyz123"], cutoff=0.9)
assert m9 == []; _ledger.append(1)

# cutoff=0.0 returns a list (admits anything similar enough to score)
m11 = difflib.get_close_matches("apple", ["xyz"], cutoff=0.0)
assert isinstance(m11, list); _ledger.append(1)

print(f"MAMBA_ASSERTION_PASS: test_difflib_get_close_matches_ops {sum(_ledger)} asserts")
