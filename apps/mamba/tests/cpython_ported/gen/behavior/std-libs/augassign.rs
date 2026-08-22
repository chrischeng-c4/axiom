use super::super::super::super::harness::*;

/// Ported from `tests/cpython/behavior/std-libs/augassign/aug_assign_test__test_in_dict.py`.
#[test]
fn test_gen_behavior_std_libs_augassign_aug_assign_test__test_in_dict() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "augassign"
# dimension = "behavior"
# case = "aug_assign_test__test_in_dict"
# subject = "cpython.test_augassign.AugAssignTest.testInDict"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_augassign.py"
# status = "filled"
# ///
# Auto-ported from CPython 3.12 test_augassign.py::AugAssignTest::testInDict
"""Auto-ported test: AugAssignTest::testInDict (CPython 3.12 oracle)."""


import unittest


# --- test body ---
x = {0: 2}
x[0] += 1
x[0] *= 2
x[0] **= 2
x[0] -= 8
x[0] //= 5
x[0] %= 3
x[0] &= 2
x[0] |= 5
x[0] ^= 1
x[0] /= 2

assert x[0] == 3.0
print("AugAssignTest::testInDict: ok")
"###);
    assert_output(&out, r###"AugAssignTest::testInDict: ok
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/augassign/aug_assign_test__test_sequences.py`.
#[test]
fn test_gen_behavior_std_libs_augassign_aug_assign_test__test_sequences() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "augassign"
# dimension = "behavior"
# case = "aug_assign_test__test_sequences"
# subject = "cpython.test_augassign.AugAssignTest.testSequences"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_augassign.py"
# status = "filled"
# ///
# Auto-ported from CPython 3.12 test_augassign.py::AugAssignTest::testSequences
"""Auto-ported test: AugAssignTest::testSequences (CPython 3.12 oracle)."""


import unittest


# --- test body ---
x = [1, 2]
x += [3, 4]
x *= 2

assert x == [1, 2, 3, 4, 1, 2, 3, 4]
x = [1, 2, 3]
y = x
x[1:2] *= 2
y[1:2] += [1]

assert x == [1, 2, 1, 2, 3]

assert x is y
print("AugAssignTest::testSequences: ok")
"###);
    assert_output(&out, r###"AugAssignTest::testSequences: ok
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/augassign/aug_assign_test__test_with_unpacking.py`.
#[test]
fn test_gen_behavior_std_libs_augassign_aug_assign_test__test_with_unpacking() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "augassign"
# dimension = "behavior"
# case = "aug_assign_test__test_with_unpacking"
# subject = "cpython.test_augassign.AugAssignTest.test_with_unpacking"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_augassign.py"
# status = "filled"
# ///
# Auto-ported from CPython 3.12 test_augassign.py::AugAssignTest::test_with_unpacking
"""Auto-ported test: AugAssignTest::test_with_unpacking (CPython 3.12 oracle)."""


import unittest


# --- test body ---

try:
    compile('x, b += 3', '<test>', 'exec')
    raise AssertionError('expected SyntaxError')
except SyntaxError:
    pass
print("AugAssignTest::test_with_unpacking: ok")
"###);
    assert_output(&out, r###"AugAssignTest::test_with_unpacking: ok
"###);
}
