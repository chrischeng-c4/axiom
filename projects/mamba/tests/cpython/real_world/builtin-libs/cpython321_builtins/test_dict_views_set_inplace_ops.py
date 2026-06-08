# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "builtin-libs"
# lib = "cpython321_builtins"
# dimension = "real_world"
# case = "test_dict_views_set_inplace_ops"
# subject = "cpython321.test_dict_views_set_inplace_ops"
# kind = "semantic"
# xfail = "CPython 3.12 seed pass; mamba promotion pending"
# mem_carveout = ""
# source = "tests/cpython/config/seeds/pass/test_dict_views_set_inplace_ops.py"
# status = "filled"
# ///
"""cpython321.test_dict_views_set_inplace_ops: execute CPython 3.12 seed test_dict_views_set_inplace_ops"""
# mamba-xfail: CPython 3.12 seed pass; mamba promotion pending
# Operational AssertionPass seed for dict views (`.keys()` /
# `.values()` / `.items()`), set in-place mutation operators (`|=`,
# `&=`, `-=`, `^=`), frozenset binary set-algebra operators, set
# predicate methods (`isdisjoint` / `issubset` / `issuperset` /
# `symmetric_difference`), and `int.to_bytes` with `signed=True` on a
# negative integer. Existing dict / set / frozenset seeds
# (test_dict_advanced_ops, test_set_method_api_ops,
# test_set_algebra_ops, test_set_mutation_ops,
# test_frozenset_set_algebra_ops) cover the binary-operator and
# basic-method surface, but skip these specific corners. mamba 0.3.60
# supports every probed form below.
#
# Surface:
#   • d.keys() / d.values() / d.items() — `len`, `in`, `sorted(_)`,
#     and `list(_)` conversion all work;
#   • set `|=` (union mutate) / `&=` (intersect mutate) /
#     `-=` (difference mutate) / `^=` (symmetric difference mutate);
#   • frozenset binary `&`, `|`, `-`, `^` against `set` RHS;
#   • set predicates: `isdisjoint`, `issubset`, `issuperset`, plus
#     `symmetric_difference` as the non-mutating form of `^`;
#   • `int.to_bytes(n, byteorder)` for positive ints, plus
#     `signed=True` for negative ints (two's-complement bytes).
_ledger: list[int] = []

# dict views — len / contains / sorted iteration
_d = {'a': 1, 'b': 2, 'c': 3}
assert len(_d.keys()) == 3; _ledger.append(1)
assert len(_d.values()) == 3; _ledger.append(1)
assert len(_d.items()) == 3; _ledger.append(1)
assert 'a' in _d.keys(); _ledger.append(1)
assert 'zzz' not in _d.keys(); _ledger.append(1)
assert sorted(_d.keys()) == ['a', 'b', 'c']; _ledger.append(1)
assert sorted(_d.values()) == [1, 2, 3]; _ledger.append(1)
assert sorted(_d.items()) == [('a', 1), ('b', 2), ('c', 3)]; _ledger.append(1)
assert sorted(list(_d.keys())) == ['a', 'b', 'c']; _ledger.append(1)
assert sorted(list(_d.values())) == [1, 2, 3]; _ledger.append(1)

# Empty dict views
_d2: dict[str, int] = {}
assert len(_d2.keys()) == 0; _ledger.append(1)
assert list(_d2.values()) == []; _ledger.append(1)
assert list(_d2.items()) == []; _ledger.append(1)

# set |= (union mutate)
_s1 = {1, 2, 3}
_s1 |= {3, 4, 5}
assert sorted(_s1) == [1, 2, 3, 4, 5]; _ledger.append(1)

# set &= (intersect mutate)
_s2 = {1, 2, 3, 4, 5}
_s2 &= {2, 3, 4}
assert sorted(_s2) == [2, 3, 4]; _ledger.append(1)

# set -= (difference mutate)
_s3 = {1, 2, 3, 4, 5}
_s3 -= {3}
assert sorted(_s3) == [1, 2, 4, 5]; _ledger.append(1)

# set -= multi-element
_s3b = {1, 2, 3, 4, 5}
_s3b -= {1, 3, 5}
assert sorted(_s3b) == [2, 4]; _ledger.append(1)

# set ^= (symmetric difference mutate)
_s4 = {1, 2, 3}
_s4 ^= {2, 3, 4}
assert sorted(_s4) == [1, 4]; _ledger.append(1)

# frozenset & set RHS
_fs = frozenset([1, 2, 3])
assert sorted(_fs & {2, 3, 4}) == [2, 3]; _ledger.append(1)

# frozenset | set RHS
assert sorted(_fs | {4, 5}) == [1, 2, 3, 4, 5]; _ledger.append(1)

# frozenset - set RHS
assert sorted(_fs - {2}) == [1, 3]; _ledger.append(1)

# frozenset ^ set RHS
assert sorted(_fs ^ {2, 3, 4}) == [1, 4]; _ledger.append(1)

# set.isdisjoint — True when no overlap
assert {1, 2, 3}.isdisjoint({4, 5}); _ledger.append(1)

# set.isdisjoint — False when overlap exists
assert not {1, 2, 3}.isdisjoint({3, 4}); _ledger.append(1)

# set.isdisjoint on empty arg always True
assert {1, 2, 3}.isdisjoint(set()); _ledger.append(1)

# set.issubset — proper subset returns True
assert {1, 2}.issubset({1, 2, 3}); _ledger.append(1)

# set.issubset — non-subset returns False
assert not {1, 2, 5}.issubset({1, 2, 3}); _ledger.append(1)

# set.issubset — equal sets are mutual subsets
assert {1, 2, 3}.issubset({1, 2, 3}); _ledger.append(1)

# set.issuperset — proper superset returns True
assert {1, 2, 3}.issuperset({1, 2}); _ledger.append(1)

# set.issuperset — non-superset returns False
assert not {1, 2}.issuperset({1, 2, 3}); _ledger.append(1)

# set.symmetric_difference (non-mutating form of ^)
assert {1, 2, 3}.symmetric_difference({2, 3, 4}) == {1, 4}; _ledger.append(1)

# int.to_bytes — positive ints big/little
assert (256).to_bytes(2, 'big') == b'\x01\x00'; _ledger.append(1)
assert (256).to_bytes(2, 'little') == b'\x00\x01'; _ledger.append(1)

# int.to_bytes — single byte boundary
assert (255).to_bytes(1, 'big') == b'\xff'; _ledger.append(1)

# int.to_bytes — 4-byte big/little
assert (1).to_bytes(4, 'big') == b'\x00\x00\x00\x01'; _ledger.append(1)
assert (1).to_bytes(4, 'little') == b'\x01\x00\x00\x00'; _ledger.append(1)

# int.to_bytes — signed=True for negative
assert (-1).to_bytes(2, 'big', signed=True) == b'\xff\xff'; _ledger.append(1)

# int.to_bytes — zero
assert (0).to_bytes(2, 'big') == b'\x00\x00'; _ledger.append(1)

print(f"MAMBA_ASSERTION_PASS: test_dict_views_set_inplace_ops {sum(_ledger)} asserts")
