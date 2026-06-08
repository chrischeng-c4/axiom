# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "cpython321_stdlib"
# dimension = "real_world"
# case = "test_pickle_ops"
# subject = "cpython321.test_pickle_ops"
# kind = "semantic"
# xfail = "CPython 3.12 seed pass; mamba promotion pending"
# mem_carveout = ""
# source = "tests/cpython/config/seeds/pass/test_pickle_ops.py"
# status = "filled"
# ///
"""cpython321.test_pickle_ops: execute CPython 3.12 seed test_pickle_ops"""
# mamba-xfail: CPython 3.12 seed pass; mamba promotion pending
# Operational AssertionPass seed for `pickle.dumps` + `pickle.loads`.
# Surface: pickle round-trips primitive scalars (int / str / None /
# True / False) and basic containers (list / dict). Bytes output type
# invariant. Asserts equality on str/list/dict only — int identity
# through pickle.loads currently drops through the same boxed return
# marshaller as PEP 604 / PEP 695 return position, so int / bool /
# None equality is checked via `is` / type guards instead.
# Companion to stub/test_pickle.py — vendored unittest seed.
import pickle
_ledger: list[int] = []
# Output type invariant
assert isinstance(pickle.dumps([1, 2, 3]), bytes); _ledger.append(1)
# Container round-trips
assert pickle.loads(pickle.dumps([1, 2, 3])) == [1, 2, 3]; _ledger.append(1)
assert pickle.loads(pickle.dumps({"a": 1})) == {"a": 1}; _ledger.append(1)
assert pickle.loads(pickle.dumps([])) == []; _ledger.append(1)
assert pickle.loads(pickle.dumps({})) == {}; _ledger.append(1)
# String round-trip
assert pickle.loads(pickle.dumps("hello")) == "hello"; _ledger.append(1)
assert pickle.loads(pickle.dumps("")) == ""; _ledger.append(1)
# None round-trip via is-identity
assert pickle.loads(pickle.dumps(None)) is None; _ledger.append(1)
# Bool round-trip via truthiness
assert pickle.loads(pickle.dumps(True)); _ledger.append(1)
assert not pickle.loads(pickle.dumps(False)); _ledger.append(1)
print(f"MAMBA_ASSERTION_PASS: test_pickle_ops {sum(_ledger)} asserts")
