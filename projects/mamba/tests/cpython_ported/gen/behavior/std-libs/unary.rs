use super::super::super::super::harness::*;

/// Ported from `tests/cpython/behavior/std-libs/unary/unary_op_test_case__test_invert.py`.
#[test]
fn test_gen_behavior_std_libs_unary_unary_op_test_case__test_invert() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "unary"
# dimension = "behavior"
# case = "unary_op_test_case__test_invert"
# subject = "cpython.test_unary.UnaryOpTestCase.test_invert"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_unary.py"
# status = "filled"
# ///
# Auto-ported from CPython 3.12 test_unary.py::UnaryOpTestCase::test_invert
"""Auto-ported test: UnaryOpTestCase::test_invert (CPython 3.12 oracle)."""


import unittest


'Test compiler changes for unary ops (+, -, ~) introduced in Python 2.2'


# --- test body ---

assert ~2 == -(2 + 1)

assert ~0 == -1

assert ~~2 == 2
print("UnaryOpTestCase::test_invert: ok")
"###);
    assert_output(&out, r###"UnaryOpTestCase::test_invert: ok
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/unary/unary_op_test_case__test_negation_of_exponentiation.py`.
#[test]
fn test_gen_behavior_std_libs_unary_unary_op_test_case__test_negation_of_exponentiation() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "unary"
# dimension = "behavior"
# case = "unary_op_test_case__test_negation_of_exponentiation"
# subject = "cpython.test_unary.UnaryOpTestCase.test_negation_of_exponentiation"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_unary.py"
# status = "filled"
# ///
# Auto-ported from CPython 3.12 test_unary.py::UnaryOpTestCase::test_negation_of_exponentiation
"""Auto-ported test: UnaryOpTestCase::test_negation_of_exponentiation (CPython 3.12 oracle)."""


import unittest


'Test compiler changes for unary ops (+, -, ~) introduced in Python 2.2'


# --- test body ---

assert -2 ** 3 == -8

assert (-2) ** 3 == -8

assert -2 ** 4 == -16

assert (-2) ** 4 == 16
print("UnaryOpTestCase::test_negation_of_exponentiation: ok")
"###);
    assert_output(&out, r###"UnaryOpTestCase::test_negation_of_exponentiation: ok
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/unary/unary_op_test_case__test_negative.py`.
#[test]
fn test_gen_behavior_std_libs_unary_unary_op_test_case__test_negative() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "unary"
# dimension = "behavior"
# case = "unary_op_test_case__test_negative"
# subject = "cpython.test_unary.UnaryOpTestCase.test_negative"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_unary.py"
# status = "filled"
# ///
# Auto-ported from CPython 3.12 test_unary.py::UnaryOpTestCase::test_negative
"""Auto-ported test: UnaryOpTestCase::test_negative (CPython 3.12 oracle)."""


import unittest


'Test compiler changes for unary ops (+, -, ~) introduced in Python 2.2'


# --- test body ---

assert -2 == 0 - 2

assert -0 == 0

assert --2 == 2

assert -2.0 == 0 - 2.0

assert -2j == 0 - 2j
print("UnaryOpTestCase::test_negative: ok")
"###);
    assert_output(&out, r###"UnaryOpTestCase::test_negative: ok
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/unary/unary_op_test_case__test_positive.py`.
#[test]
fn test_gen_behavior_std_libs_unary_unary_op_test_case__test_positive() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "unary"
# dimension = "behavior"
# case = "unary_op_test_case__test_positive"
# subject = "cpython.test_unary.UnaryOpTestCase.test_positive"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_unary.py"
# status = "filled"
# ///
# Auto-ported from CPython 3.12 test_unary.py::UnaryOpTestCase::test_positive
"""Auto-ported test: UnaryOpTestCase::test_positive (CPython 3.12 oracle)."""


import unittest


'Test compiler changes for unary ops (+, -, ~) introduced in Python 2.2'


# --- test body ---

assert +2 == 2

assert +0 == 0

assert ++2 == 2

assert +2.0 == 2.0

assert +2j == 2j
print("UnaryOpTestCase::test_positive: ok")
"###);
    assert_output(&out, r###"UnaryOpTestCase::test_positive: ok
"###);
}
