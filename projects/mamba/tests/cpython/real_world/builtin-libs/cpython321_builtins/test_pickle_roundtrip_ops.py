# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "builtin-libs"
# lib = "cpython321_builtins"
# dimension = "real_world"
# case = "test_pickle_roundtrip_ops"
# subject = "cpython321.test_pickle_roundtrip_ops"
# kind = "semantic"
# xfail = "CPython 3.12 seed pass; mamba promotion pending"
# mem_carveout = ""
# source = "tests/cpython/config/seeds/pass/test_pickle_roundtrip_ops.py"
# status = "filled"
# ///
"""cpython321.test_pickle_roundtrip_ops: execute CPython 3.12 seed test_pickle_roundtrip_ops"""
# mamba-xfail: CPython 3.12 seed pass; mamba promotion pending
# Operational AssertionPass seed for `pickle.dumps` / `pickle.loads`
# round-trip serialization. Surface: `pickle.dumps(obj)` produces a
# `bytes` blob that `pickle.loads` parses back into a structurally
# equal object. Round-trip is verified for the standard atomic types
# (int including negative, str, float, bool, None), the standard
# container types (list, tuple, dict including nested), and the empty
# forms of each container. Large strings round-trip without
# truncation. The optional `protocol=` kwarg selects a wire format
# (protocol=0 is the ASCII-safe legacy format).
import pickle
_ledger: list[int] = []

# Atomic types
assert pickle.loads(pickle.dumps(42)) == 42; _ledger.append(1)
assert pickle.loads(pickle.dumps(-99)) == -99; _ledger.append(1)
assert pickle.loads(pickle.dumps("hello")) == "hello"; _ledger.append(1)
assert pickle.loads(pickle.dumps(3.14)) == 3.14; _ledger.append(1)
assert pickle.loads(pickle.dumps(True)) == True; _ledger.append(1)
assert pickle.loads(pickle.dumps(None)) is None; _ledger.append(1)

# Container types
assert pickle.loads(pickle.dumps([1, 2, 3])) == [1, 2, 3]; _ledger.append(1)
assert pickle.loads(pickle.dumps((1, 2, 3))) == (1, 2, 3); _ledger.append(1)
assert pickle.loads(pickle.dumps({"a": 1})) == {"a": 1}; _ledger.append(1)

# Nested container — heterogeneous depth
assert pickle.loads(pickle.dumps({"x": [1, 2, {"y": 3}]})) == {"x": [1, 2, {"y": 3}]}; _ledger.append(1)

# Empty container forms
assert pickle.loads(pickle.dumps([])) == []; _ledger.append(1)
assert pickle.loads(pickle.dumps({})) == {}; _ledger.append(1)
assert pickle.loads(pickle.dumps(())) == (); _ledger.append(1)

# Long string round-trip
assert pickle.loads(pickle.dumps("a" * 100)) == "a" * 100; _ledger.append(1)

# Wire format invariants
assert isinstance(pickle.dumps(1), bytes); _ledger.append(1)
assert isinstance(pickle.dumps("x"), bytes); _ledger.append(1)

# Explicit protocol selector
assert pickle.loads(pickle.dumps(42, protocol=0)) == 42; _ledger.append(1)

print(f"MAMBA_ASSERTION_PASS: test_pickle_roundtrip_ops {sum(_ledger)} asserts")
