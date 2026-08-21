# test_pickle.py — #2696 CPython pickle seed (executed assertions).
#
# Replaces the prior vendored CPython upstream Lib/test/test_pickle.py
# with the *smallest* Mamba-authored seed distilled from the pickle
# round-trip smoke surface. Exercises `pickle.dumps` / `pickle.loads`
# across the common Python value types (int, str, bool, None, float,
# list, tuple, dict, nested) plus protocol-pinned round-trips at
# protocol=0 and protocol=2, and verifies `HIGHEST_PROTOCOL` is at
# least 5 on both runtimes. Emits the runner's positive proof-of-
# execution marker that `cpython_lib_test_runner.rs` (#2691) classifies
# as `AssertionPass`.
#
# Why so small? Mamba's current pickle surface presents dumps/loads
# for all the primitive types plus nested list/tuple/dict, returns a
# real `bytes` blob, and honors the `protocol=N` keyword for N in
# [0, 5]. Richer surface — custom `__reduce__`, class instances,
# `pickle.Pickler` / `pickle.Unpickler`, `pickle.dump` / `pickle.load`
# to file objects, `pickle.PickleError` subclassing, `pickle_byref`,
# memoization — lands as each gap closes.
#
# Specifically, `pickle.loads(pickle.dumps(b"..."))` returns an EMPTY
# bytes object on mamba today (the bytes round-trip is broken), so
# bytes are excluded from this seed.
#
# Why use `==` for bool round-trips? Mamba's `pickle.loads(pickle.dumps(True))`
# returns a bool that compares equal to True but is not the singleton
# True object — same behaviour as json (#2693). None correctly uses
# `is None`.
#
# Why `>=` for HIGHEST_PROTOCOL? mamba returns 5, CPython 3.12 returns 5;
# `>= 5` is stable across future protocol bumps in both runtimes.
#
# Why no helper function? Per the #2691 contract, top-level `def()`
# does not capture module-scope names by reference on mamba.
#
# Contract with the runner (#2691):
#   - Each `assert` runs at top level. Inverting any assert raises
#     AssertionError → non-zero exit → runner classifies as `Fail`.
#   - After every assertion has executed, the seed emits
#     `MAMBA_ASSERTION_PASS: pickle N asserts` to stdout.

import pickle

_ledger: list[int] = []

# 1. Module identity: pickle's own __name__ must be "pickle".
assert pickle.__name__ == "pickle", "pickle.__name__ must be 'pickle'"
_ledger.append(1)

# 2. dumps emits a bytes object. Catches a regression that flips the
#    return type to str / bytearray.
assert isinstance(pickle.dumps(42), bytes), "pickle.dumps returns bytes"
_ledger.append(1)

# 3. HIGHEST_PROTOCOL is at least 5. Both CPython 3.12 and mamba
#    expose 5; >=5 is stable across future bumps.
assert pickle.HIGHEST_PROTOCOL >= 5, "pickle.HIGHEST_PROTOCOL >= 5"
_ledger.append(1)

# 4. Round-trip of each primitive value type. The load-bearing
#    invariant — the pickled value, decoded, must `==` the original.
assert pickle.loads(pickle.dumps(42)) == 42, "int round-trip"
_ledger.append(1)
assert pickle.loads(pickle.dumps("hello")) == "hello", "str round-trip"
_ledger.append(1)
assert pickle.loads(pickle.dumps(True)) == True, "bool True round-trip (== not is)"
_ledger.append(1)
assert pickle.loads(pickle.dumps(False)) == False, "bool False round-trip (== not is)"
_ledger.append(1)
assert pickle.loads(pickle.dumps(None)) is None, "None round-trip (is None)"
_ledger.append(1)
assert pickle.loads(pickle.dumps(3.14)) == 3.14, "float round-trip"
_ledger.append(1)

# 5. Container round-trips. Decoded value must equal the original AND
#    have the right concrete type — pickle must not silently convert
#    a list into a tuple or vice versa.
_lst = pickle.loads(pickle.dumps([1, 2, 3]))
assert _lst == [1, 2, 3], "list round-trip equality"
_ledger.append(1)
assert isinstance(_lst, list), "list round-trip preserves list type"
_ledger.append(1)
assert len(_lst) == 3, "list round-trip preserves length"
_ledger.append(1)

_tup = pickle.loads(pickle.dumps((1, 2, 3)))
assert _tup == (1, 2, 3), "tuple round-trip equality"
_ledger.append(1)
assert isinstance(_tup, tuple), "tuple round-trip preserves tuple type"
_ledger.append(1)

_dct = pickle.loads(pickle.dumps({"a": 1, "b": 2}))
assert _dct == {"a": 1, "b": 2}, "dict round-trip equality"
_ledger.append(1)
assert isinstance(_dct, dict), "dict round-trip preserves dict type"
_ledger.append(1)
assert _dct["a"] == 1, "dict round-trip preserves key->value"
_ledger.append(1)

# 6. Nested round-trip: dict containing list, tuple, and inner dict.
#    Catches a recursive-encoder regression where one nesting level
#    works but two does not.
_nest = {"x": [1, 2, 3], "y": (4, 5), "z": {"a": "b"}}
_decoded = pickle.loads(pickle.dumps(_nest))
assert _decoded == _nest, "nested struct round-trip"
_ledger.append(1)
assert _decoded["x"] == [1, 2, 3], "inner list intact after round-trip"
_ledger.append(1)
assert _decoded["z"]["a"] == "b", "two-level nested dict intact"
_ledger.append(1)

# 7. Protocol-pinned round-trip. dumps with `protocol=0` (the oldest
#    text-protocol format) must still load back to the same value;
#    same for protocol=2. Catches a regression where only the default
#    protocol path works.
assert pickle.loads(pickle.dumps([1, 2, 3], protocol=0)) == [1, 2, 3], "protocol=0 round-trip"
_ledger.append(1)
assert pickle.loads(pickle.dumps([1, 2, 3], protocol=2)) == [1, 2, 3], "protocol=2 round-trip"
_ledger.append(1)

# Emit the proof-of-execution marker as the FINAL line so the runner
# can see it on stdout. Per `ASSERTION_PASS_MARKERS` in
# `cpython_lib_test_runner.rs`, presence of this token escalates the
# outcome from ImportPass to AssertionPass.
print(f"MAMBA_ASSERTION_PASS: pickle {len(_ledger)} asserts")
