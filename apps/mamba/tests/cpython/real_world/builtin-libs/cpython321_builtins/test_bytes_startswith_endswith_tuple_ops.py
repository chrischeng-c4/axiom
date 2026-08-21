# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "builtin-libs"
# lib = "cpython321_builtins"
# dimension = "real_world"
# case = "test_bytes_startswith_endswith_tuple_ops"
# subject = "cpython321.test_bytes_startswith_endswith_tuple_ops"
# kind = "semantic"
# xfail = "CPython 3.12 seed pass; mamba promotion pending"
# mem_carveout = ""
# source = "tests/cpython/config/seeds/pass/test_bytes_startswith_endswith_tuple_ops.py"
# status = "filled"
# ///
"""cpython321.test_bytes_startswith_endswith_tuple_ops: execute CPython 3.12 seed test_bytes_startswith_endswith_tuple_ops"""
# mamba-xfail: CPython 3.12 seed pass; mamba promotion pending
# Operational AssertionPass seed for bytes/bytearray method surfaces
# not covered by `test_bytes_ops`, `test_bytes_method_extras_ops`,
# `test_bytes_operators_slicing_ops`, `test_bytearray_ops`, or
# `test_bytes_hex_encoding_ops`. This seed asserts: bytes.startswith
# and bytes.endswith accept a TUPLE of candidate prefixes / suffixes
# returning True if ANY element matches; bytes.startswith /
# bytes.endswith accept start/end positional arguments (slicing the
# match window); bytes.split accepts an explicit `maxsplit` cap;
# bytes.replace accepts a `count` cap that limits the number of
# substitutions; bytes.find / .rfind return -1 for a missing needle
# even when start/end shrink the search window; bytes.find with
# explicit start positional advances the search beyond an earlier
# match; bytes.count with start/end positional clamps the count to
# the slice window; the empty-bytes prefix/suffix matches every
# byte string; `bytes.replace(b"x", b"")` deletes matches without
# leaving placeholder runs.
_ledger: list[int] = []

# startswith with tuple of prefixes
assert b"hello".startswith((b"he", b"wo")); _ledger.append(1)
assert b"hello".startswith((b"wo", b"he")); _ledger.append(1)
assert b"world".startswith((b"wo", b"he")); _ledger.append(1)
assert not b"banana".startswith((b"app", b"or")); _ledger.append(1)
assert b"hello".startswith((b"hello", b"hey")); _ledger.append(1)

# endswith with tuple of suffixes
assert b"hello".endswith((b"lo", b"world")); _ledger.append(1)
assert b"hello".endswith((b"world", b"lo")); _ledger.append(1)
assert b"file.txt".endswith((b".py", b".txt")); _ledger.append(1)
assert b"file.py".endswith((b".py", b".txt")); _ledger.append(1)
assert not b"file.md".endswith((b".py", b".txt")); _ledger.append(1)

# Empty-bytes prefix/suffix matches every byte string
assert b"hello".startswith(b""); _ledger.append(1)
assert b"hello".endswith(b""); _ledger.append(1)
assert b"".startswith(b""); _ledger.append(1)
assert b"".endswith(b""); _ledger.append(1)

# startswith / endswith with start positional
assert b"hello world".startswith(b"world", 6); _ledger.append(1)
assert b"hello world".startswith(b"hello", 0); _ledger.append(1)
assert not b"hello world".startswith(b"world", 0); _ledger.append(1)
assert b"hello world".endswith(b"hello", 0, 5); _ledger.append(1)
assert b"hello world".endswith(b"world", 0, 11); _ledger.append(1)
assert not b"hello world".endswith(b"hello", 0, 11); _ledger.append(1)

# split with explicit maxsplit cap
assert b"a,b,c,d".split(b",", 1) == [b"a", b"b,c,d"]; _ledger.append(1)
assert b"a,b,c,d".split(b",", 2) == [b"a", b"b", b"c,d"]; _ledger.append(1)
assert b"a,b,c,d".split(b",", 0) == [b"a,b,c,d"]; _ledger.append(1)
assert b"a,b,c,d".split(b",", -1) == [b"a", b"b", b"c", b"d"]; _ledger.append(1)
assert b"a,b,c".split(b",", 99) == [b"a", b"b", b"c"]; _ledger.append(1)

# replace with count cap
assert b"aaa".replace(b"a", b"x", 1) == b"xaa"; _ledger.append(1)
assert b"aaa".replace(b"a", b"x", 2) == b"xxa"; _ledger.append(1)
assert b"aaa".replace(b"a", b"x", 0) == b"aaa"; _ledger.append(1)
assert b"hello".replace(b"l", b"L", 1) == b"heLlo"; _ledger.append(1)
assert b"banana".replace(b"a", b"_", 2) == b"b_n_na"; _ledger.append(1)

# replace with empty replacement deletes matches
assert b"banana".replace(b"a", b"") == b"bnn"; _ledger.append(1)
assert b"hello".replace(b"l", b"") == b"heo"; _ledger.append(1)
assert b"abcabc".replace(b"abc", b"") == b""; _ledger.append(1)

# find / rfind shrunk window returns -1 when match falls outside
assert b"hello world".find(b"hello") == 0; _ledger.append(1)
assert b"hello world".find(b"hello", 1) == -1; _ledger.append(1)
assert b"hello world".rfind(b"hello") == 0; _ledger.append(1)
assert b"hello world".rfind(b"hello", 1) == -1; _ledger.append(1)
assert b"hello world".find(b"world") == 6; _ledger.append(1)
assert b"hello world".find(b"world", 0, 5) == -1; _ledger.append(1)

# find with start positional advances past earlier match
assert b"abcabc".find(b"abc") == 0; _ledger.append(1)
assert b"abcabc".find(b"abc", 1) == 3; _ledger.append(1)
assert b"abcabc".rfind(b"abc") == 3; _ledger.append(1)
assert b"abcabc".rfind(b"abc", 0, 5) == 0; _ledger.append(1)

# count with start/end clamps count to slice window
assert b"banana".count(b"a") == 3; _ledger.append(1)
assert b"banana".count(b"a", 2) == 2; _ledger.append(1)
assert b"banana".count(b"a", 0, 3) == 1; _ledger.append(1)
assert b"banana".count(b"na") == 2; _ledger.append(1)
assert b"banana".count(b"na", 3) == 1; _ledger.append(1)

# split with no separator collapses runs of whitespace
assert b"a  b   c".split() == [b"a", b"b", b"c"]; _ledger.append(1)
assert b"  leading".split() == [b"leading"]; _ledger.append(1)
assert b"trailing  ".split() == [b"trailing"]; _ledger.append(1)
assert b"".split() == []; _ledger.append(1)

# Empty-needle find returns 0 (matches at start)
assert b"hello".find(b"") == 0; _ledger.append(1)
# Single-byte replace with multi-byte expansion
assert b"abc".replace(b"a", b"AA") == b"AAbc"; _ledger.append(1)
assert b"abc".replace(b"b", b"XY") == b"aXYc"; _ledger.append(1)
assert b"aaa".replace(b"a", b"AA") == b"AAAAAA"; _ledger.append(1)
# Replace nothing when needle missing
assert b"hello".replace(b"z", b"Z") == b"hello"; _ledger.append(1)

print(f"MAMBA_ASSERTION_PASS: test_bytes_startswith_endswith_tuple_ops {sum(_ledger)} asserts")
