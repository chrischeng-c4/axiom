# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "builtin-libs"
# lib = "cpython321_builtins"
# dimension = "real_world"
# case = "test_chainmap_ops"
# subject = "cpython321.test_chainmap_ops"
# kind = "semantic"
# xfail = "CPython 3.12 seed pass; mamba promotion pending"
# mem_carveout = ""
# source = "tests/cpython/config/seeds/pass/test_chainmap_ops.py"
# status = "filled"
# ///
"""cpython321.test_chainmap_ops: execute CPython 3.12 seed test_chainmap_ops"""
# mamba-xfail: CPython 3.12 seed pass; mamba promotion pending
# Operational AssertionPass seed for `collections.ChainMap`.
# Surface: lookup precedence (first dict in the chain wins on
# collisions), len (union of keys across chained dicts), key
# iteration includes every distinct key from the chain.
from collections import ChainMap
_ledger: list[int] = []
a = {"x": 1, "y": 2}
b = {"y": 99, "z": 3}
cm = ChainMap(a, b)
# First-dict precedence on overlapping keys
assert cm["y"] == 2; _ledger.append(1)
# Keys unique to either dict are reachable
assert cm["x"] == 1; _ledger.append(1)
assert cm["z"] == 3; _ledger.append(1)
# len() counts distinct keys across the union (x, y, z = 3)
assert len(cm) == 3; _ledger.append(1)
# Iteration covers every distinct key (order is implementation
# defined, so compare as a set)
assert set(list(cm)) == {"x", "y", "z"}; _ledger.append(1)
# Reversing the chain inverts precedence on overlaps
cm2 = ChainMap(b, a)
assert cm2["y"] == 99; _ledger.append(1)
# Reversed chain still resolves keys unique to either dict
assert cm2["x"] == 1; _ledger.append(1)
assert cm2["z"] == 3; _ledger.append(1)
print(f"MAMBA_ASSERTION_PASS: test_chainmap_ops {sum(_ledger)} asserts")
