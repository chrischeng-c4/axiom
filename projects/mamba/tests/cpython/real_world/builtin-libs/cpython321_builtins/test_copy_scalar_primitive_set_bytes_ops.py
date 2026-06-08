# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "builtin-libs"
# lib = "cpython321_builtins"
# dimension = "real_world"
# case = "test_copy_scalar_primitive_set_bytes_ops"
# subject = "cpython321.test_copy_scalar_primitive_set_bytes_ops"
# kind = "semantic"
# xfail = "CPython 3.12 seed pass; mamba promotion pending"
# mem_carveout = ""
# source = "tests/cpython/config/seeds/pass/test_copy_scalar_primitive_set_bytes_ops.py"
# status = "filled"
# ///
"""cpython321.test_copy_scalar_primitive_set_bytes_ops: execute CPython 3.12 seed test_copy_scalar_primitive_set_bytes_ops"""
# mamba-xfail: CPython 3.12 seed pass; mamba promotion pending
# Operational AssertionPass seed for the `copy` stdlib module's
# scalar / primitive / set / bytes surface — the matching subset
# that the existing `test_copy_shallow_deep_ops.py` does NOT
# already exercise. That file covers the canonical list / dict /
# tuple shallow/deep semantics. This seed fills the orthogonal
# value-side of the `copy.copy` / `copy.deepcopy` contract:
# immutable scalars (`int`, `float`, `str`, `bool`, `None`,
# `bytes`), the canonical immutable set type (`frozenset`), the
# mutable `set` type, the mutable container identity test
# (`copy(x) is not x` for mutables), and deepcopy roundtrip for
# the same scalar / primitive / set family. Same matching subset
# between mamba and CPython.
#
# Surface (the matching subset between mamba and CPython):
#   • copy.copy(int / float / str / bool / None / bytes) returns
#     the equal value (immutables, possibly same identity);
#   • copy.copy({int_set}) and copy.copy(frozenset(...)) return
#     equal sets;
#   • copy.copy on a mutable container produces a NEW top-level
#     object (`copy.copy(x) is not x` is True for list/dict/set);
#   • copy.copy on an immutable container/value MAY share identity
#     but value-equals;
#   • copy.deepcopy(scalars/primitives) round-trips by value;
#   • copy.deepcopy(set / frozenset) round-trips by value;
#   • copy.deepcopy(nested dict-with-list-and-tuple-and-bytes)
#     round-trips by value, with the inner mutable list
#     independent from the source.
import copy
_ledger: list[int] = []

# copy.copy on int — value-equal
assert copy.copy(0) == 0; _ledger.append(1)
assert copy.copy(1) == 1; _ledger.append(1)
assert copy.copy(42) == 42; _ledger.append(1)
assert copy.copy(-1) == -1; _ledger.append(1)
assert copy.copy(2**30) == 2**30; _ledger.append(1)

# copy.copy on float — value-equal
assert copy.copy(0.0) == 0.0; _ledger.append(1)
assert copy.copy(1.5) == 1.5; _ledger.append(1)
assert copy.copy(-3.14) == -3.14; _ledger.append(1)

# copy.copy on str — value-equal
assert copy.copy("") == ""; _ledger.append(1)
assert copy.copy("a") == "a"; _ledger.append(1)
assert copy.copy("hello") == "hello"; _ledger.append(1)
assert copy.copy("with spaces") == "with spaces"; _ledger.append(1)

# copy.copy on bool — value-equal
assert copy.copy(True) == True; _ledger.append(1)
assert copy.copy(False) == False; _ledger.append(1)
assert copy.copy(True) != False; _ledger.append(1)
assert copy.copy(False) != True; _ledger.append(1)

# copy.copy on None — identity preserved (sentinel)
assert copy.copy(None) is None; _ledger.append(1)
assert copy.copy(None) == None; _ledger.append(1)

# copy.copy on bytes — value-equal
assert copy.copy(b"") == b""; _ledger.append(1)
assert copy.copy(b"abc") == b"abc"; _ledger.append(1)
assert copy.copy(b"\x00\xff") == b"\x00\xff"; _ledger.append(1)

# copy.copy on frozenset — value-equal
assert copy.copy(frozenset()) == frozenset(); _ledger.append(1)
assert copy.copy(frozenset([1])) == frozenset([1]); _ledger.append(1)
assert copy.copy(frozenset([1, 2, 3])) == frozenset([1, 2, 3]); _ledger.append(1)
assert copy.copy(frozenset(["a", "b"])) == frozenset(["a", "b"]); _ledger.append(1)

# copy.copy on mutable set — equal AND new top-level object
_s1 = {1, 2, 3}
_s2 = copy.copy(_s1)
assert _s1 == _s2; _ledger.append(1)
assert _s2 == {1, 2, 3}; _ledger.append(1)
_s2.add(99)
assert 99 in _s2; _ledger.append(1)
assert 99 not in _s1; _ledger.append(1)

# copy.copy on empty set
_es = set()
_es2 = copy.copy(_es)
assert _es2 == set(); _ledger.append(1)
_es2.add(1)
assert _es == set(); _ledger.append(1)
assert _es2 == {1}; _ledger.append(1)

# Mutable container identity contract — copy.copy MUST produce
# a different top-level object for list/dict/set
_l = [1, 2, 3]
assert copy.copy(_l) is not _l; _ledger.append(1)
_d = {"a": 1}
assert copy.copy(_d) is not _d; _ledger.append(1)
_st = {1, 2}
assert copy.copy(_st) is not _st; _ledger.append(1)

# copy.deepcopy on scalars — value-equal
assert copy.deepcopy(0) == 0; _ledger.append(1)
assert copy.deepcopy(42) == 42; _ledger.append(1)
assert copy.deepcopy(3.14) == 3.14; _ledger.append(1)
assert copy.deepcopy("hello") == "hello"; _ledger.append(1)
assert copy.deepcopy(True) == True; _ledger.append(1)
assert copy.deepcopy(False) == False; _ledger.append(1)
assert copy.deepcopy(None) is None; _ledger.append(1)
assert copy.deepcopy(b"abc") == b"abc"; _ledger.append(1)

# copy.deepcopy on set / frozenset — value-equal
assert copy.deepcopy(set()) == set(); _ledger.append(1)
assert copy.deepcopy({1, 2, 3}) == {1, 2, 3}; _ledger.append(1)
assert copy.deepcopy(frozenset()) == frozenset(); _ledger.append(1)
assert copy.deepcopy(frozenset([1, 2])) == frozenset([1, 2]); _ledger.append(1)

# copy.deepcopy mutable-set identity — equal but independent
_ds = {1, 2, 3}
_ds2 = copy.deepcopy(_ds)
assert _ds == _ds2; _ledger.append(1)
_ds2.add(99)
assert 99 not in _ds; _ledger.append(1)
assert 99 in _ds2; _ledger.append(1)

# copy.deepcopy on tuple of immutables — equal
assert copy.deepcopy(()) == (); _ledger.append(1)
assert copy.deepcopy((1, 2, 3)) == (1, 2, 3); _ledger.append(1)
assert copy.deepcopy(("a", "b")) == ("a", "b"); _ledger.append(1)
assert copy.deepcopy((1, "a", None, True)) == (1, "a", None, True); _ledger.append(1)

# copy.deepcopy on tuple containing list — outer tuple equal,
# inner list is a NEW list (mutation through deep copy local)
_tl = ([1, 2], [3, 4])
_tl2 = copy.deepcopy(_tl)
assert _tl2 == ([1, 2], [3, 4]); _ledger.append(1)
_tl2[0].append(99)
assert _tl == ([1, 2], [3, 4]); _ledger.append(1)
assert _tl2 == ([1, 2, 99], [3, 4]); _ledger.append(1)

# copy.deepcopy of nested dict-with-mixed — list / tuple / bytes
_mix = {"items": [1, 2, 3], "shape": (4, 5), "blob": b"abc"}
_mix2 = copy.deepcopy(_mix)
assert _mix == _mix2; _ledger.append(1)
_mix2["items"].append(99)
assert _mix["items"] == [1, 2, 3]; _ledger.append(1)
assert _mix2["items"] == [1, 2, 3, 99]; _ledger.append(1)
# Bytes survive intact
assert _mix2["blob"] == b"abc"; _ledger.append(1)
# Tuple survives intact
assert _mix2["shape"] == (4, 5); _ledger.append(1)

# Round-trip — copy.copy of a copy returns equal value
assert copy.copy(copy.copy(42)) == 42; _ledger.append(1)
assert copy.copy(copy.copy("hello")) == "hello"; _ledger.append(1)
assert copy.copy(copy.copy(b"abc")) == b"abc"; _ledger.append(1)
assert copy.copy(copy.copy(None)) is None; _ledger.append(1)
assert copy.copy(copy.copy(frozenset([1, 2]))) == frozenset([1, 2]); _ledger.append(1)

# Round-trip — copy.deepcopy of a deepcopy returns equal value
assert copy.deepcopy(copy.deepcopy(42)) == 42; _ledger.append(1)
assert copy.deepcopy(copy.deepcopy("hello")) == "hello"; _ledger.append(1)
assert copy.deepcopy(copy.deepcopy({1, 2, 3})) == {1, 2, 3}; _ledger.append(1)

# Mixed copy/deepcopy round-trip
assert copy.copy(copy.deepcopy(42)) == 42; _ledger.append(1)
assert copy.deepcopy(copy.copy("hello")) == "hello"; _ledger.append(1)

# Module-level discipline — `copy` is a real module with the two
# canonical entry points
assert callable(copy.copy); _ledger.append(1)
assert callable(copy.deepcopy); _ledger.append(1)
assert hasattr(copy, "copy"); _ledger.append(1)
assert hasattr(copy, "deepcopy"); _ledger.append(1)
assert copy.__name__ == "copy"; _ledger.append(1)

print(f"MAMBA_ASSERTION_PASS: test_copy_scalar_primitive_set_bytes_ops {sum(_ledger)} asserts")
