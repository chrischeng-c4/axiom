# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "cpython321_stdlib"
# dimension = "real_world"
# case = "test_copy"
# subject = "cpython321.test_copy"
# kind = "semantic"
# xfail = "CPython 3.12 seed pass; mamba promotion pending"
# mem_carveout = ""
# source = "tests/cpython/config/seeds/pass/test_copy.py"
# status = "filled"
# ///
"""cpython321.test_copy: execute CPython 3.12 seed test_copy"""
# mamba-xfail: CPython 3.12 seed pass; mamba promotion pending
# test_copy.py — #2827 CPython copy seed (executed assertions).
#
# Replaces the prior vendored CPython upstream Lib/test/test_copy.py
# (ranked `Stub` — the seed previously reached unittest.main() but
# the assertions never executed) with a Mamba-authored seed distilled
# from the copy / deepcopy / Error surface. Exercises shallow and
# deep copy of the three core container shapes (list, dict, tuple)
# and the SHARED-vs-NOT-SHARED inner-reference invariant — the
# load-bearing claim of the module. Emits the runner's positive
# proof-of-execution marker that `cpython_lib_test_runner.rs` (#2691)
# classifies as `AssertionPass`.
#
# Why so small? Mamba's current copy surface (copy_mod.rs) presents
# copy.copy / copy.deepcopy / copy.Error and produces the same
# is-not-source / equal-to-source / shared-inner-for-shallow /
# fresh-inner-for-deep semantics as CPython on list/dict/tuple.
# Richer surface — `__copy__` / `__deepcopy__` / `__reduce_ex__`
# hooks on user classes, deepcopy memo cycle-detection — lands as
# each gap closes.
#
# Why a small object graph? Per #2827 acceptance: "Fixture keeps
# object graph small." So we stick to 3-element lists / 2-key dicts.
#
# Why no helper function? Per the #2691 contract, top-level `def()`
# does not capture module-scope names by reference on mamba.
#
# Contract with the runner (#2691):
#   - Each `assert` runs at top level. Inverting any assert raises
#     AssertionError → non-zero exit → runner classifies as `Fail`.
#   - After every assertion has executed, the seed emits
#     `MAMBA_ASSERTION_PASS: copy N asserts` to stdout.

import copy

_ledger: list[int] = []

# 1. Module identity + public surface.
assert copy.__name__ == "copy", "copy.__name__ must be 'copy'"
_ledger.append(1)
assert hasattr(copy, "copy"), "copy must expose copy()"
_ledger.append(1)
assert hasattr(copy, "deepcopy"), "copy must expose deepcopy()"
_ledger.append(1)
assert hasattr(copy, "Error"), "copy must expose Error"
_ledger.append(1)

# 2. Shallow copy of a nested list. The outer list must be a fresh
#    object that compares equal but is NOT identical, and the inner
#    list must be SHARED (this is what makes it "shallow").
_src_list = [1, 2, [3, 4]]
_sh_list = copy.copy(_src_list)
assert _sh_list == _src_list, "shallow copy of list compares equal"
_ledger.append(1)
assert _sh_list is not _src_list, "shallow copy returns a fresh list (not the same object)"
_ledger.append(1)
assert _sh_list[2] is _src_list[2], "shallow copy SHARES the inner list reference"
_ledger.append(1)

# 3. Deep copy of the same nested list. The outer list must be a
#    fresh object that compares equal but is NOT identical, AND the
#    inner list must be a FRESH copy (the load-bearing deepcopy
#    invariant — and the #2827 acceptance pivot).
_src_list2 = [1, 2, [3, 4]]
_dp_list = copy.deepcopy(_src_list2)
assert _dp_list == _src_list2, "deep copy of list compares equal"
_ledger.append(1)
assert _dp_list is not _src_list2, "deep copy returns a fresh list (not the same object)"
_ledger.append(1)
assert _dp_list[2] is not _src_list2[2], "deep copy does NOT share the inner list (fresh copy)"
_ledger.append(1)

# 4. Mutation invariant: mutating the deep copy's inner list must NOT
#    affect the source. This is what users actually rely on (it's why
#    they reach for deepcopy instead of copy).
_dp_list[2].append(99)
assert _src_list2 == [1, 2, [3, 4]], "mutating deep copy's inner list does NOT touch source"
_ledger.append(1)
assert _dp_list == [1, 2, [3, 4, 99]], "deep copy's inner list reflects the mutation"
_ledger.append(1)

# 5. Shallow copy of a dict with a list value. Same invariants as the
#    list case — outer dict fresh, inner list shared.
_src_dict = {"a": [1, 2], "b": 3}
_sh_dict = copy.copy(_src_dict)
assert _sh_dict == _src_dict, "shallow copy of dict compares equal"
_ledger.append(1)
assert _sh_dict is not _src_dict, "shallow copy returns a fresh dict"
_ledger.append(1)
assert _sh_dict["a"] is _src_dict["a"], "shallow copy of dict SHARES list value"
_ledger.append(1)

# 6. Deep copy of the same dict — inner list must be a fresh copy.
_src_dict2 = {"a": [1, 2], "b": 3}
_dp_dict = copy.deepcopy(_src_dict2)
assert _dp_dict == _src_dict2, "deep copy of dict compares equal"
_ledger.append(1)
assert _dp_dict is not _src_dict2, "deep copy returns a fresh dict"
_ledger.append(1)
assert _dp_dict["a"] is not _src_dict2["a"], "deep copy of dict does NOT share list value"
_ledger.append(1)

# 7. Tuple deep copy. Tuples are immutable but may contain mutable
#    inner objects. The inner list reference must be fresh in the
#    deep copy.
_src_tuple = (1, 2, [3, 4])
_dp_tuple = copy.deepcopy(_src_tuple)
assert _dp_tuple == _src_tuple, "deep copy of tuple compares equal"
_ledger.append(1)
assert _dp_tuple[2] is not _src_tuple[2], "deep copy of tuple does NOT share inner list"
_ledger.append(1)

# Emit the proof-of-execution marker as the FINAL line so the runner
# can see it on stdout. Per `ASSERTION_PASS_MARKERS` in
# `cpython_lib_test_runner.rs`, presence of this token escalates the
# outcome from ImportPass to AssertionPass.
print(f"MAMBA_ASSERTION_PASS: copy {len(_ledger)} asserts")
