# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "cpython321_stdlib"
# dimension = "real_world"
# case = "test_pickle_nested_ops"
# subject = "cpython321.test_pickle_nested_ops"
# kind = "semantic"
# xfail = "CPython 3.12 seed pass; mamba promotion pending"
# mem_carveout = ""
# source = "tests/cpython/config/seeds/pass/test_pickle_nested_ops.py"
# status = "filled"
# ///
"""cpython321.test_pickle_nested_ops: execute CPython 3.12 seed test_pickle_nested_ops"""
# mamba-xfail: CPython 3.12 seed pass; mamba promotion pending
# Operational AssertionPass seed for pickle surfaces beyond
# test_pickle_ops (which covers int/str/list/dict/None/True/False/
# bytes-output-type round-trips).
# Surface: tuple round-trips preserve the tuple type (not promoted
# to a list); float round-trips preserve the exact bit pattern for
# values that are exactly representable in IEEE 754; deeply-nested
# dict-of-list-of-dict-of-list round-trips; list-of-dict and dict-
# of-list shapes round-trip without losing order; an all-types-mixed
# list survives pickle dump / load; pickle.dumps always returns
# `bytes` (never a memoryview / bytearray); empty containers and
# single-element containers round-trip.
import pickle
_ledger: list[int] = []

# Tuple round-trip preserves the tuple type
assert pickle.loads(pickle.dumps((1, 2, 3))) == (1, 2, 3); _ledger.append(1)
assert pickle.loads(pickle.dumps(())) == (); _ledger.append(1)
assert pickle.loads(pickle.dumps((42,))) == (42,); _ledger.append(1)
# A round-tripped tuple is still a tuple (not promoted to a list)
assert type(pickle.loads(pickle.dumps((1, 2)))).__name__ == "tuple"; _ledger.append(1)

# Float round-trip for exact-binary values (1.0, 0.5, 0.25, 3.14)
assert pickle.loads(pickle.dumps(1.0)) == 1.0; _ledger.append(1)
assert pickle.loads(pickle.dumps(0.5)) == 0.5; _ledger.append(1)
assert pickle.loads(pickle.dumps(0.25)) == 0.25; _ledger.append(1)
assert pickle.loads(pickle.dumps(3.14)) == 3.14; _ledger.append(1)
assert pickle.loads(pickle.dumps(-1.5)) == -1.5; _ledger.append(1)
assert pickle.loads(pickle.dumps(0.0)) == 0.0; _ledger.append(1)

# Nested dict-of-list round-trips
groups = {"odds": [1, 3, 5], "evens": [2, 4, 6]}
assert pickle.loads(pickle.dumps(groups)) == groups; _ledger.append(1)

# Nested list-of-dict round-trips
records = [{"name": "alice", "age": 30}, {"name": "bob", "age": 25}]
assert pickle.loads(pickle.dumps(records)) == records; _ledger.append(1)

# Deeply-nested mixed structure
deep = {"items": [1, 2, {"nested": True, "list": [10, 20, [30, 40]]}]}
assert pickle.loads(pickle.dumps(deep)) == deep; _ledger.append(1)

# All-types-mixed list round-trip
mixed = [1, "two", 3.5, True, None, [4, 5], {"k": "v"}, (6, 7)]
assert pickle.loads(pickle.dumps(mixed)) == mixed; _ledger.append(1)

# pickle.dumps always returns `bytes`
assert type(pickle.dumps([1, 2, 3])).__name__ == "bytes"; _ledger.append(1)
assert type(pickle.dumps({})).__name__ == "bytes"; _ledger.append(1)
assert type(pickle.dumps("hello")).__name__ == "bytes"; _ledger.append(1)
assert type(pickle.dumps(42)).__name__ == "bytes"; _ledger.append(1)
assert type(pickle.dumps(None)).__name__ == "bytes"; _ledger.append(1)

# Empty containers round-trip
assert pickle.loads(pickle.dumps([])) == []; _ledger.append(1)
assert pickle.loads(pickle.dumps({})) == {}; _ledger.append(1)
assert pickle.loads(pickle.dumps(())) == (); _ledger.append(1)
assert pickle.loads(pickle.dumps("")) == ""; _ledger.append(1)

# Single-element containers round-trip
assert pickle.loads(pickle.dumps([42])) == [42]; _ledger.append(1)
assert pickle.loads(pickle.dumps({"k": 1})) == {"k": 1}; _ledger.append(1)
assert pickle.loads(pickle.dumps((42,))) == (42,); _ledger.append(1)

# Multi-level nesting round-trips
two_deep = [[1, 2], [3, 4], [5, 6]]
assert pickle.loads(pickle.dumps(two_deep)) == two_deep; _ledger.append(1)

three_deep = [[[1, 2], [3, 4]], [[5, 6], [7, 8]]]
assert pickle.loads(pickle.dumps(three_deep)) == three_deep; _ledger.append(1)

# Dict-of-dict round-trip
dod = {"a": {"x": 1, "y": 2}, "b": {"z": 3}}
assert pickle.loads(pickle.dumps(dod)) == dod; _ledger.append(1)

# Round-trip preserves equality even after a write/read cycle
data = {"items": [1, 2, 3], "meta": {"count": 3}}
assert pickle.loads(pickle.dumps(data)) == data; _ledger.append(1)
# And a second round-trip is still equal
assert pickle.loads(pickle.dumps(pickle.loads(pickle.dumps(data)))) == data; _ledger.append(1)
print(f"MAMBA_ASSERTION_PASS: test_pickle_nested_ops {sum(_ledger)} asserts")
