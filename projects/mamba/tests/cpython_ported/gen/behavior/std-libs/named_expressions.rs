use super::super::super::super::harness::*;

/// Ported from `tests/cpython/behavior/std-libs/named_expressions/named_expression_assignment_test__test_named_expression_assignment_01.py`.
#[test]
fn test_gen_behavior_std_libs_named_expressions_named_expression_assignment_test__test_named_expression_assignment_01() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "named_expressions"
# dimension = "behavior"
# case = "named_expression_assignment_test__test_named_expression_assignment_01"
# subject = "cpython.test_named_expressions.NamedExpressionAssignmentTest.test_named_expression_assignment_01"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_named_expressions.py"
# status = "filled"
# ///
# Auto-ported from CPython 3.12 test_named_expressions.py::NamedExpressionAssignmentTest::test_named_expression_assignment_01
"""Auto-ported test: NamedExpressionAssignmentTest::test_named_expression_assignment_01 (CPython 3.12 oracle)."""


import unittest


GLOBAL_VAR = None


# --- test body ---
(a := 10)

assert a == 10
print("NamedExpressionAssignmentTest::test_named_expression_assignment_01: ok")
"###);
    assert_output(&out, r###"NamedExpressionAssignmentTest::test_named_expression_assignment_01: ok
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/named_expressions/named_expression_assignment_test__test_named_expression_assignment_02.py`.
#[test]
fn test_gen_behavior_std_libs_named_expressions_named_expression_assignment_test__test_named_expression_assignment_02() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "named_expressions"
# dimension = "behavior"
# case = "named_expression_assignment_test__test_named_expression_assignment_02"
# subject = "cpython.test_named_expressions.NamedExpressionAssignmentTest.test_named_expression_assignment_02"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_named_expressions.py"
# status = "filled"
# ///
# Auto-ported from CPython 3.12 test_named_expressions.py::NamedExpressionAssignmentTest::test_named_expression_assignment_02
"""Auto-ported test: NamedExpressionAssignmentTest::test_named_expression_assignment_02 (CPython 3.12 oracle)."""


import unittest


GLOBAL_VAR = None


# --- test body ---
a = 20
(a := a)

assert a == 20
print("NamedExpressionAssignmentTest::test_named_expression_assignment_02: ok")
"###);
    assert_output(&out, r###"NamedExpressionAssignmentTest::test_named_expression_assignment_02: ok
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/named_expressions/named_expression_assignment_test__test_named_expression_assignment_03.py`.
#[test]
fn test_gen_behavior_std_libs_named_expressions_named_expression_assignment_test__test_named_expression_assignment_03() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "named_expressions"
# dimension = "behavior"
# case = "named_expression_assignment_test__test_named_expression_assignment_03"
# subject = "cpython.test_named_expressions.NamedExpressionAssignmentTest.test_named_expression_assignment_03"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_named_expressions.py"
# status = "filled"
# ///
# Auto-ported from CPython 3.12 test_named_expressions.py::NamedExpressionAssignmentTest::test_named_expression_assignment_03
"""Auto-ported test: NamedExpressionAssignmentTest::test_named_expression_assignment_03 (CPython 3.12 oracle)."""


import unittest


GLOBAL_VAR = None


# --- test body ---
(total := (1 + 2))

assert total == 3
print("NamedExpressionAssignmentTest::test_named_expression_assignment_03: ok")
"###);
    assert_output(&out, r###"NamedExpressionAssignmentTest::test_named_expression_assignment_03: ok
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/named_expressions/named_expression_assignment_test__test_named_expression_assignment_04.py`.
#[test]
fn test_gen_behavior_std_libs_named_expressions_named_expression_assignment_test__test_named_expression_assignment_04() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "named_expressions"
# dimension = "behavior"
# case = "named_expression_assignment_test__test_named_expression_assignment_04"
# subject = "cpython.test_named_expressions.NamedExpressionAssignmentTest.test_named_expression_assignment_04"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_named_expressions.py"
# status = "filled"
# ///
# Auto-ported from CPython 3.12 test_named_expressions.py::NamedExpressionAssignmentTest::test_named_expression_assignment_04
"""Auto-ported test: NamedExpressionAssignmentTest::test_named_expression_assignment_04 (CPython 3.12 oracle)."""


import unittest


GLOBAL_VAR = None


# --- test body ---
(info := (1, 2, 3))

assert info == (1, 2, 3)
print("NamedExpressionAssignmentTest::test_named_expression_assignment_04: ok")
"###);
    assert_output(&out, r###"NamedExpressionAssignmentTest::test_named_expression_assignment_04: ok
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/named_expressions/named_expression_assignment_test__test_named_expression_assignment_05.py`.
#[test]
fn test_gen_behavior_std_libs_named_expressions_named_expression_assignment_test__test_named_expression_assignment_05() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "named_expressions"
# dimension = "behavior"
# case = "named_expression_assignment_test__test_named_expression_assignment_05"
# subject = "cpython.test_named_expressions.NamedExpressionAssignmentTest.test_named_expression_assignment_05"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_named_expressions.py"
# status = "filled"
# ///
# Auto-ported from CPython 3.12 test_named_expressions.py::NamedExpressionAssignmentTest::test_named_expression_assignment_05
"""Auto-ported test: NamedExpressionAssignmentTest::test_named_expression_assignment_05 (CPython 3.12 oracle)."""


import unittest


GLOBAL_VAR = None


# --- test body ---
((x := 1), 2)

assert x == 1
print("NamedExpressionAssignmentTest::test_named_expression_assignment_05: ok")
"###);
    assert_output(&out, r###"NamedExpressionAssignmentTest::test_named_expression_assignment_05: ok
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/named_expressions/named_expression_assignment_test__test_named_expression_assignment_06.py`.
#[test]
fn test_gen_behavior_std_libs_named_expressions_named_expression_assignment_test__test_named_expression_assignment_06() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "named_expressions"
# dimension = "behavior"
# case = "named_expression_assignment_test__test_named_expression_assignment_06"
# subject = "cpython.test_named_expressions.NamedExpressionAssignmentTest.test_named_expression_assignment_06"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_named_expressions.py"
# status = "filled"
# ///
# Auto-ported from CPython 3.12 test_named_expressions.py::NamedExpressionAssignmentTest::test_named_expression_assignment_06
"""Auto-ported test: NamedExpressionAssignmentTest::test_named_expression_assignment_06 (CPython 3.12 oracle)."""


import unittest


GLOBAL_VAR = None


# --- test body ---
(z := (y := (x := 0)))

assert x == 0

assert y == 0

assert z == 0
print("NamedExpressionAssignmentTest::test_named_expression_assignment_06: ok")
"###);
    assert_output(&out, r###"NamedExpressionAssignmentTest::test_named_expression_assignment_06: ok
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/named_expressions/named_expression_assignment_test__test_named_expression_assignment_07.py`.
#[test]
fn test_gen_behavior_std_libs_named_expressions_named_expression_assignment_test__test_named_expression_assignment_07() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "named_expressions"
# dimension = "behavior"
# case = "named_expression_assignment_test__test_named_expression_assignment_07"
# subject = "cpython.test_named_expressions.NamedExpressionAssignmentTest.test_named_expression_assignment_07"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_named_expressions.py"
# status = "filled"
# ///
# Auto-ported from CPython 3.12 test_named_expressions.py::NamedExpressionAssignmentTest::test_named_expression_assignment_07
"""Auto-ported test: NamedExpressionAssignmentTest::test_named_expression_assignment_07 (CPython 3.12 oracle)."""


import unittest


GLOBAL_VAR = None


# --- test body ---
(loc := (1, 2))

assert loc == (1, 2)
print("NamedExpressionAssignmentTest::test_named_expression_assignment_07: ok")
"###);
    assert_output(&out, r###"NamedExpressionAssignmentTest::test_named_expression_assignment_07: ok
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/named_expressions/named_expression_assignment_test__test_named_expression_assignment_08.py`.
#[test]
fn test_gen_behavior_std_libs_named_expressions_named_expression_assignment_test__test_named_expression_assignment_08() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "named_expressions"
# dimension = "behavior"
# case = "named_expression_assignment_test__test_named_expression_assignment_08"
# subject = "cpython.test_named_expressions.NamedExpressionAssignmentTest.test_named_expression_assignment_08"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_named_expressions.py"
# status = "filled"
# ///
# Auto-ported from CPython 3.12 test_named_expressions.py::NamedExpressionAssignmentTest::test_named_expression_assignment_08
"""Auto-ported test: NamedExpressionAssignmentTest::test_named_expression_assignment_08 (CPython 3.12 oracle)."""


import unittest


GLOBAL_VAR = None


# --- test body ---
if (spam := 'eggs'):

    assert spam == 'eggs'
else:

    raise AssertionError('variable was not assigned using named expression')
print("NamedExpressionAssignmentTest::test_named_expression_assignment_08: ok")
"###);
    assert_output(&out, r###"NamedExpressionAssignmentTest::test_named_expression_assignment_08: ok
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/named_expressions/named_expression_assignment_test__test_named_expression_assignment_09.py`.
#[test]
fn test_gen_behavior_std_libs_named_expressions_named_expression_assignment_test__test_named_expression_assignment_09() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "named_expressions"
# dimension = "behavior"
# case = "named_expression_assignment_test__test_named_expression_assignment_09"
# subject = "cpython.test_named_expressions.NamedExpressionAssignmentTest.test_named_expression_assignment_09"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_named_expressions.py"
# status = "filled"
# ///
# Auto-ported from CPython 3.12 test_named_expressions.py::NamedExpressionAssignmentTest::test_named_expression_assignment_09
"""Auto-ported test: NamedExpressionAssignmentTest::test_named_expression_assignment_09 (CPython 3.12 oracle)."""


import unittest


GLOBAL_VAR = None


# --- test body ---
if True and (spam := True):

    assert spam
else:

    raise AssertionError('variable was not assigned using named expression')
print("NamedExpressionAssignmentTest::test_named_expression_assignment_09: ok")
"###);
    assert_output(&out, r###"NamedExpressionAssignmentTest::test_named_expression_assignment_09: ok
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/named_expressions/named_expression_assignment_test__test_named_expression_assignment_10.py`.
#[test]
fn test_gen_behavior_std_libs_named_expressions_named_expression_assignment_test__test_named_expression_assignment_10() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "named_expressions"
# dimension = "behavior"
# case = "named_expression_assignment_test__test_named_expression_assignment_10"
# subject = "cpython.test_named_expressions.NamedExpressionAssignmentTest.test_named_expression_assignment_10"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_named_expressions.py"
# status = "filled"
# ///
# Auto-ported from CPython 3.12 test_named_expressions.py::NamedExpressionAssignmentTest::test_named_expression_assignment_10
"""Auto-ported test: NamedExpressionAssignmentTest::test_named_expression_assignment_10 (CPython 3.12 oracle)."""


import unittest


GLOBAL_VAR = None


# --- test body ---
if (match := 10) == 10:

    assert match == 10
else:

    raise AssertionError('variable was not assigned using named expression')
print("NamedExpressionAssignmentTest::test_named_expression_assignment_10: ok")
"###);
    assert_output(&out, r###"NamedExpressionAssignmentTest::test_named_expression_assignment_10: ok
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/named_expressions/named_expression_assignment_test__test_named_expression_assignment_11.py`.
#[test]
fn test_gen_behavior_std_libs_named_expressions_named_expression_assignment_test__test_named_expression_assignment_11() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "named_expressions"
# dimension = "behavior"
# case = "named_expression_assignment_test__test_named_expression_assignment_11"
# subject = "cpython.test_named_expressions.NamedExpressionAssignmentTest.test_named_expression_assignment_11"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_named_expressions.py"
# status = "filled"
# ///
# Auto-ported from CPython 3.12 test_named_expressions.py::NamedExpressionAssignmentTest::test_named_expression_assignment_11
"""Auto-ported test: NamedExpressionAssignmentTest::test_named_expression_assignment_11 (CPython 3.12 oracle)."""


import unittest


GLOBAL_VAR = None


# --- test body ---
def spam(a):
    return a
input_data = [1, 2, 3]
res = [(x, y, x / y) for x in input_data if (y := spam(x)) > 0]

assert res == [(1, 1, 1.0), (2, 2, 1.0), (3, 3, 1.0)]
print("NamedExpressionAssignmentTest::test_named_expression_assignment_11: ok")
"###);
    assert_output(&out, r###"NamedExpressionAssignmentTest::test_named_expression_assignment_11: ok
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/named_expressions/named_expression_assignment_test__test_named_expression_assignment_12.py`.
#[test]
fn test_gen_behavior_std_libs_named_expressions_named_expression_assignment_test__test_named_expression_assignment_12() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "named_expressions"
# dimension = "behavior"
# case = "named_expression_assignment_test__test_named_expression_assignment_12"
# subject = "cpython.test_named_expressions.NamedExpressionAssignmentTest.test_named_expression_assignment_12"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_named_expressions.py"
# status = "filled"
# ///
# Auto-ported from CPython 3.12 test_named_expressions.py::NamedExpressionAssignmentTest::test_named_expression_assignment_12
"""Auto-ported test: NamedExpressionAssignmentTest::test_named_expression_assignment_12 (CPython 3.12 oracle)."""


import unittest


GLOBAL_VAR = None


# --- test body ---
def spam(a):
    return a
res = [[(y := spam(x)), x / y] for x in range(1, 5)]

assert res == [[1, 1.0], [2, 1.0], [3, 1.0], [4, 1.0]]
print("NamedExpressionAssignmentTest::test_named_expression_assignment_12: ok")
"###);
    assert_output(&out, r###"NamedExpressionAssignmentTest::test_named_expression_assignment_12: ok
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/named_expressions/named_expression_assignment_test__test_named_expression_assignment_13.py`.
#[test]
fn test_gen_behavior_std_libs_named_expressions_named_expression_assignment_test__test_named_expression_assignment_13() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "named_expressions"
# dimension = "behavior"
# case = "named_expression_assignment_test__test_named_expression_assignment_13"
# subject = "cpython.test_named_expressions.NamedExpressionAssignmentTest.test_named_expression_assignment_13"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_named_expressions.py"
# status = "filled"
# ///
# Auto-ported from CPython 3.12 test_named_expressions.py::NamedExpressionAssignmentTest::test_named_expression_assignment_13
"""Auto-ported test: NamedExpressionAssignmentTest::test_named_expression_assignment_13 (CPython 3.12 oracle)."""


import unittest


GLOBAL_VAR = None


# --- test body ---
length = len((lines := [1, 2]))

assert length == 2

assert lines == [1, 2]
print("NamedExpressionAssignmentTest::test_named_expression_assignment_13: ok")
"###);
    assert_output(&out, r###"NamedExpressionAssignmentTest::test_named_expression_assignment_13: ok
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/named_expressions/named_expression_assignment_test__test_named_expression_assignment_14.py`.
#[test]
fn test_gen_behavior_std_libs_named_expressions_named_expression_assignment_test__test_named_expression_assignment_14() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "named_expressions"
# dimension = "behavior"
# case = "named_expression_assignment_test__test_named_expression_assignment_14"
# subject = "cpython.test_named_expressions.NamedExpressionAssignmentTest.test_named_expression_assignment_14"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_named_expressions.py"
# status = "filled"
# ///
# Auto-ported from CPython 3.12 test_named_expressions.py::NamedExpressionAssignmentTest::test_named_expression_assignment_14
"""Auto-ported test: NamedExpressionAssignmentTest::test_named_expression_assignment_14 (CPython 3.12 oracle)."""


import unittest


GLOBAL_VAR = None


# --- test body ---
"""
        Where all variables are positive integers, and a is at least as large
        as the n'th root of x, this algorithm returns the floor of the n'th
        root of x (and roughly doubling the number of accurate bits per
        iteration):
        """
a = 9
n = 2
x = 3
while a > (d := (x // a ** (n - 1))):
    a = ((n - 1) * a + d) // n

assert a == 1
print("NamedExpressionAssignmentTest::test_named_expression_assignment_14: ok")
"###);
    assert_output(&out, r###"NamedExpressionAssignmentTest::test_named_expression_assignment_14: ok
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/named_expressions/named_expression_assignment_test__test_named_expression_assignment_15.py`.
#[test]
fn test_gen_behavior_std_libs_named_expressions_named_expression_assignment_test__test_named_expression_assignment_15() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "named_expressions"
# dimension = "behavior"
# case = "named_expression_assignment_test__test_named_expression_assignment_15"
# subject = "cpython.test_named_expressions.NamedExpressionAssignmentTest.test_named_expression_assignment_15"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_named_expressions.py"
# status = "filled"
# ///
# Auto-ported from CPython 3.12 test_named_expressions.py::NamedExpressionAssignmentTest::test_named_expression_assignment_15
"""Auto-ported test: NamedExpressionAssignmentTest::test_named_expression_assignment_15 (CPython 3.12 oracle)."""


import unittest


GLOBAL_VAR = None


# --- test body ---
while (a := False):

    raise AssertionError('While body executed')

assert a == False
print("NamedExpressionAssignmentTest::test_named_expression_assignment_15: ok")
"###);
    assert_output(&out, r###"NamedExpressionAssignmentTest::test_named_expression_assignment_15: ok
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/named_expressions/named_expression_assignment_test__test_named_expression_assignment_17.py`.
#[test]
fn test_gen_behavior_std_libs_named_expressions_named_expression_assignment_test__test_named_expression_assignment_17() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "named_expressions"
# dimension = "behavior"
# case = "named_expression_assignment_test__test_named_expression_assignment_17"
# subject = "cpython.test_named_expressions.NamedExpressionAssignmentTest.test_named_expression_assignment_17"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_named_expressions.py"
# status = "filled"
# ///
# Auto-ported from CPython 3.12 test_named_expressions.py::NamedExpressionAssignmentTest::test_named_expression_assignment_17
"""Auto-ported test: NamedExpressionAssignmentTest::test_named_expression_assignment_17 (CPython 3.12 oracle)."""


import unittest


GLOBAL_VAR = None


# --- test body ---
a = [1]
element = a[(b := 0)]

assert b == 0

assert element == a[0]
print("NamedExpressionAssignmentTest::test_named_expression_assignment_17: ok")
"###);
    assert_output(&out, r###"NamedExpressionAssignmentTest::test_named_expression_assignment_17: ok
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/named_expressions/named_expression_assignment_test__test_named_expression_assignment_18.py`.
#[test]
fn test_gen_behavior_std_libs_named_expressions_named_expression_assignment_test__test_named_expression_assignment_18() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "named_expressions"
# dimension = "behavior"
# case = "named_expression_assignment_test__test_named_expression_assignment_18"
# subject = "cpython.test_named_expressions.NamedExpressionAssignmentTest.test_named_expression_assignment_18"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_named_expressions.py"
# status = "filled"
# ///
# Auto-ported from CPython 3.12 test_named_expressions.py::NamedExpressionAssignmentTest::test_named_expression_assignment_18
"""Auto-ported test: NamedExpressionAssignmentTest::test_named_expression_assignment_18 (CPython 3.12 oracle)."""


import unittest


GLOBAL_VAR = None


# --- test body ---
class TwoDimensionalList:

    def __init__(self, two_dimensional_list):
        self.two_dimensional_list = two_dimensional_list

    def __getitem__(self, index):
        return self.two_dimensional_list[index[0]][index[1]]
a = TwoDimensionalList([[1], [2]])
element = a[(b := 0), (c := 0)]

assert b == 0

assert c == 0

assert element == a.two_dimensional_list[b][c]
print("NamedExpressionAssignmentTest::test_named_expression_assignment_18: ok")
"###);
    assert_output(&out, r###"NamedExpressionAssignmentTest::test_named_expression_assignment_18: ok
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/named_expressions/named_expression_invalid_test__test_named_expression_invalid_08.py`.
#[test]
fn test_gen_behavior_std_libs_named_expressions_named_expression_invalid_test__test_named_expression_invalid_08() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "named_expressions"
# dimension = "behavior"
# case = "named_expression_invalid_test__test_named_expression_invalid_08"
# subject = "cpython.test_named_expressions.NamedExpressionInvalidTest.test_named_expression_invalid_08"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_named_expressions.py"
# status = "filled"
# ///
# Auto-ported from CPython 3.12 test_named_expressions.py::NamedExpressionInvalidTest::test_named_expression_invalid_08
"""Auto-ported test: NamedExpressionInvalidTest::test_named_expression_invalid_08 (CPython 3.12 oracle)."""


import unittest


GLOBAL_VAR = None


# --- test body ---
code = 'def spam(a: b := 42 = 5): pass'
try:
    exec(code, {}, {})
    raise AssertionError('expected SyntaxError')
except SyntaxError as _aR_e:
    import re as _re_aR
    assert _re_aR.search('invalid syntax', str(_aR_e))
print("NamedExpressionInvalidTest::test_named_expression_invalid_08: ok")
"###);
    assert_output(&out, r###"NamedExpressionInvalidTest::test_named_expression_invalid_08: ok
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/named_expressions/named_expression_invalid_test__test_named_expression_invalid_16.py`.
#[test]
fn test_gen_behavior_std_libs_named_expressions_named_expression_invalid_test__test_named_expression_invalid_16() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "named_expressions"
# dimension = "behavior"
# case = "named_expression_invalid_test__test_named_expression_invalid_16"
# subject = "cpython.test_named_expressions.NamedExpressionInvalidTest.test_named_expression_invalid_16"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_named_expressions.py"
# status = "filled"
# ///
# Auto-ported from CPython 3.12 test_named_expressions.py::NamedExpressionInvalidTest::test_named_expression_invalid_16
"""Auto-ported test: NamedExpressionInvalidTest::test_named_expression_invalid_16 (CPython 3.12 oracle)."""


import unittest


GLOBAL_VAR = None


# --- test body ---
code = '[i + 1 for i in i := [1,2]]'
try:
    exec(code, {}, {})
    raise AssertionError('expected SyntaxError')
except SyntaxError as _aR_e:
    import re as _re_aR
    assert _re_aR.search('invalid syntax', str(_aR_e))
print("NamedExpressionInvalidTest::test_named_expression_invalid_16: ok")
"###);
    assert_output(&out, r###"NamedExpressionInvalidTest::test_named_expression_invalid_16: ok
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/named_expressions/named_expression_scope_test__test_named_expression_global_scope_uc5521b4.py`.
#[test]
fn test_gen_behavior_std_libs_named_expressions_named_expression_scope_test__test_named_expression_global_scope_uc5521b4() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "named_expressions"
# dimension = "behavior"
# case = "named_expression_scope_test__test_named_expression_global_scope_uc5521b4"
# subject = "cpython.test_named_expressions.NamedExpressionScopeTest.test_named_expression_global_scope"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_named_expressions.py"
# status = "filled"
# ///
sentinel = object()
global GLOBAL_VAR

def f():
    global GLOBAL_VAR
    [(GLOBAL_VAR := sentinel) for _ in range(1)]
    assert GLOBAL_VAR == sentinel
try:
    f()
    assert GLOBAL_VAR == sentinel
finally:
    GLOBAL_VAR = None

print("NamedExpressionScopeTest::test_named_expression_global_scope: ok")
"###);
    assert_output(&out, r###"NamedExpressionScopeTest::test_named_expression_global_scope: ok
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/named_expressions/named_expression_scope_test__test_named_expression_nonlocal_scope_no_nonlocal_keyword_ucfb3f42.py`.
#[test]
fn test_gen_behavior_std_libs_named_expressions_named_expression_scope_test__test_named_expression_nonlocal_scope_no_nonlocal_keyword_ucfb3f42() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "named_expressions"
# dimension = "behavior"
# case = "named_expression_scope_test__test_named_expression_nonlocal_scope_no_nonlocal_keyword_ucfb3f42"
# subject = "cpython.test_named_expressions.NamedExpressionScopeTest.test_named_expression_nonlocal_scope_no_nonlocal_keyword"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_named_expressions.py"
# status = "filled"
# ///
sentinel = object()

def f():
    nonlocal_var = None

    def g():
        [(nonlocal_var := sentinel) for _ in range(1)]
    g()
    assert nonlocal_var == None
f()

print("NamedExpressionScopeTest::test_named_expression_nonlocal_scope_no_nonlocal_keyword: ok")
"###);
    assert_output(&out, r###"NamedExpressionScopeTest::test_named_expression_nonlocal_scope_no_nonlocal_keyword: ok
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/named_expressions/named_expression_scope_test__test_named_expression_nonlocal_scope_uc0e372a.py`.
#[test]
fn test_gen_behavior_std_libs_named_expressions_named_expression_scope_test__test_named_expression_nonlocal_scope_uc0e372a() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "named_expressions"
# dimension = "behavior"
# case = "named_expression_scope_test__test_named_expression_nonlocal_scope_uc0e372a"
# subject = "cpython.test_named_expressions.NamedExpressionScopeTest.test_named_expression_nonlocal_scope"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_named_expressions.py"
# status = "filled"
# ///
sentinel = object()

def f():
    nonlocal_var = None

    def g():
        nonlocal nonlocal_var
        [(nonlocal_var := sentinel) for _ in range(1)]
    g()
    assert nonlocal_var == sentinel
f()

print("NamedExpressionScopeTest::test_named_expression_nonlocal_scope: ok")
"###);
    assert_output(&out, r###"NamedExpressionScopeTest::test_named_expression_nonlocal_scope: ok
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/named_expressions/named_expression_scope_test__test_named_expression_scope_02.py`.
#[test]
fn test_gen_behavior_std_libs_named_expressions_named_expression_scope_test__test_named_expression_scope_02() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "named_expressions"
# dimension = "behavior"
# case = "named_expression_scope_test__test_named_expression_scope_02"
# subject = "cpython.test_named_expressions.NamedExpressionScopeTest.test_named_expression_scope_02"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_named_expressions.py"
# status = "filled"
# ///
# Auto-ported from CPython 3.12 test_named_expressions.py::NamedExpressionScopeTest::test_named_expression_scope_02
"""Auto-ported test: NamedExpressionScopeTest::test_named_expression_scope_02 (CPython 3.12 oracle)."""


import unittest


GLOBAL_VAR = None


# --- test body ---
total = 0
partial_sums = [(total := (total + v)) for v in range(5)]

assert partial_sums == [0, 1, 3, 6, 10]

assert total == 10
print("NamedExpressionScopeTest::test_named_expression_scope_02: ok")
"###);
    assert_output(&out, r###"NamedExpressionScopeTest::test_named_expression_scope_02: ok
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/named_expressions/named_expression_scope_test__test_named_expression_scope_03.py`.
#[test]
fn test_gen_behavior_std_libs_named_expressions_named_expression_scope_test__test_named_expression_scope_03() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "named_expressions"
# dimension = "behavior"
# case = "named_expression_scope_test__test_named_expression_scope_03"
# subject = "cpython.test_named_expressions.NamedExpressionScopeTest.test_named_expression_scope_03"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_named_expressions.py"
# status = "filled"
# ///
# Auto-ported from CPython 3.12 test_named_expressions.py::NamedExpressionScopeTest::test_named_expression_scope_03
"""Auto-ported test: NamedExpressionScopeTest::test_named_expression_scope_03 (CPython 3.12 oracle)."""


import unittest


GLOBAL_VAR = None


# --- test body ---
containsOne = any(((lastNum := num) == 1 for num in [1, 2, 3]))

assert containsOne

assert lastNum == 1
print("NamedExpressionScopeTest::test_named_expression_scope_03: ok")
"###);
    assert_output(&out, r###"NamedExpressionScopeTest::test_named_expression_scope_03: ok
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/named_expressions/named_expression_scope_test__test_named_expression_scope_04.py`.
#[test]
fn test_gen_behavior_std_libs_named_expressions_named_expression_scope_test__test_named_expression_scope_04() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "named_expressions"
# dimension = "behavior"
# case = "named_expression_scope_test__test_named_expression_scope_04"
# subject = "cpython.test_named_expressions.NamedExpressionScopeTest.test_named_expression_scope_04"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_named_expressions.py"
# status = "filled"
# ///
# Auto-ported from CPython 3.12 test_named_expressions.py::NamedExpressionScopeTest::test_named_expression_scope_04
"""Auto-ported test: NamedExpressionScopeTest::test_named_expression_scope_04 (CPython 3.12 oracle)."""


import unittest


GLOBAL_VAR = None


# --- test body ---
def spam(a):
    return a
res = [[(y := spam(x)), x / y] for x in range(1, 5)]

assert y == 4
print("NamedExpressionScopeTest::test_named_expression_scope_04: ok")
"###);
    assert_output(&out, r###"NamedExpressionScopeTest::test_named_expression_scope_04: ok
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/named_expressions/named_expression_scope_test__test_named_expression_scope_05.py`.
#[test]
fn test_gen_behavior_std_libs_named_expressions_named_expression_scope_test__test_named_expression_scope_05() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "named_expressions"
# dimension = "behavior"
# case = "named_expression_scope_test__test_named_expression_scope_05"
# subject = "cpython.test_named_expressions.NamedExpressionScopeTest.test_named_expression_scope_05"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_named_expressions.py"
# status = "filled"
# ///
# Auto-ported from CPython 3.12 test_named_expressions.py::NamedExpressionScopeTest::test_named_expression_scope_05
"""Auto-ported test: NamedExpressionScopeTest::test_named_expression_scope_05 (CPython 3.12 oracle)."""


import unittest


GLOBAL_VAR = None


# --- test body ---
def spam(a):
    return a
input_data = [1, 2, 3]
res = [(x, y, x / y) for x in input_data if (y := spam(x)) > 0]

assert res == [(1, 1, 1.0), (2, 2, 1.0), (3, 3, 1.0)]

assert y == 3
print("NamedExpressionScopeTest::test_named_expression_scope_05: ok")
"###);
    assert_output(&out, r###"NamedExpressionScopeTest::test_named_expression_scope_05: ok
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/named_expressions/named_expression_scope_test__test_named_expression_scope_07.py`.
#[test]
fn test_gen_behavior_std_libs_named_expressions_named_expression_scope_test__test_named_expression_scope_07() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "named_expressions"
# dimension = "behavior"
# case = "named_expression_scope_test__test_named_expression_scope_07"
# subject = "cpython.test_named_expressions.NamedExpressionScopeTest.test_named_expression_scope_07"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_named_expressions.py"
# status = "filled"
# ///
# Auto-ported from CPython 3.12 test_named_expressions.py::NamedExpressionScopeTest::test_named_expression_scope_07
"""Auto-ported test: NamedExpressionScopeTest::test_named_expression_scope_07 (CPython 3.12 oracle)."""


import unittest


GLOBAL_VAR = None


# --- test body ---
len((lines := [1, 2]))

assert lines == [1, 2]
print("NamedExpressionScopeTest::test_named_expression_scope_07: ok")
"###);
    assert_output(&out, r###"NamedExpressionScopeTest::test_named_expression_scope_07: ok
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/named_expressions/named_expression_scope_test__test_named_expression_scope_17.py`.
#[test]
fn test_gen_behavior_std_libs_named_expressions_named_expression_scope_test__test_named_expression_scope_17() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "named_expressions"
# dimension = "behavior"
# case = "named_expression_scope_test__test_named_expression_scope_17"
# subject = "cpython.test_named_expressions.NamedExpressionScopeTest.test_named_expression_scope_17"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_named_expressions.py"
# status = "filled"
# ///
# Auto-ported from CPython 3.12 test_named_expressions.py::NamedExpressionScopeTest::test_named_expression_scope_17
"""Auto-ported test: NamedExpressionScopeTest::test_named_expression_scope_17 (CPython 3.12 oracle)."""


import unittest


GLOBAL_VAR = None


# --- test body ---
b = 0
res = [(b := (i + b)) for i in range(5)]

assert res == [0, 1, 3, 6, 10]

assert b == 10
print("NamedExpressionScopeTest::test_named_expression_scope_17: ok")
"###);
    assert_output(&out, r###"NamedExpressionScopeTest::test_named_expression_scope_17: ok
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/named_expressions/named_expression_scope_test__test_named_expression_scope_18.py`.
#[test]
fn test_gen_behavior_std_libs_named_expressions_named_expression_scope_test__test_named_expression_scope_18() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "named_expressions"
# dimension = "behavior"
# case = "named_expression_scope_test__test_named_expression_scope_18"
# subject = "cpython.test_named_expressions.NamedExpressionScopeTest.test_named_expression_scope_18"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_named_expressions.py"
# status = "filled"
# ///
# Auto-ported from CPython 3.12 test_named_expressions.py::NamedExpressionScopeTest::test_named_expression_scope_18
"""Auto-ported test: NamedExpressionScopeTest::test_named_expression_scope_18 (CPython 3.12 oracle)."""


import unittest


GLOBAL_VAR = None


# --- test body ---
def spam(a):
    return a
res = spam((b := 2))

assert res == 2

assert b == 2
print("NamedExpressionScopeTest::test_named_expression_scope_18: ok")
"###);
    assert_output(&out, r###"NamedExpressionScopeTest::test_named_expression_scope_18: ok
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/named_expressions/named_expression_scope_test__test_named_expression_scope_19.py`.
#[test]
fn test_gen_behavior_std_libs_named_expressions_named_expression_scope_test__test_named_expression_scope_19() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "named_expressions"
# dimension = "behavior"
# case = "named_expression_scope_test__test_named_expression_scope_19"
# subject = "cpython.test_named_expressions.NamedExpressionScopeTest.test_named_expression_scope_19"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_named_expressions.py"
# status = "filled"
# ///
# Auto-ported from CPython 3.12 test_named_expressions.py::NamedExpressionScopeTest::test_named_expression_scope_19
"""Auto-ported test: NamedExpressionScopeTest::test_named_expression_scope_19 (CPython 3.12 oracle)."""


import unittest


GLOBAL_VAR = None


# --- test body ---
def spam(a):
    return a
res = spam((b := 2))

assert res == 2

assert b == 2
print("NamedExpressionScopeTest::test_named_expression_scope_19: ok")
"###);
    assert_output(&out, r###"NamedExpressionScopeTest::test_named_expression_scope_19: ok
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/named_expressions/named_expression_scope_test__test_named_expression_scope_20.py`.
#[test]
fn test_gen_behavior_std_libs_named_expressions_named_expression_scope_test__test_named_expression_scope_20() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "named_expressions"
# dimension = "behavior"
# case = "named_expression_scope_test__test_named_expression_scope_20"
# subject = "cpython.test_named_expressions.NamedExpressionScopeTest.test_named_expression_scope_20"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_named_expressions.py"
# status = "filled"
# ///
# Auto-ported from CPython 3.12 test_named_expressions.py::NamedExpressionScopeTest::test_named_expression_scope_20
"""Auto-ported test: NamedExpressionScopeTest::test_named_expression_scope_20 (CPython 3.12 oracle)."""


import unittest


GLOBAL_VAR = None


# --- test body ---
def spam(a):
    return a
res = spam(a=(b := 2))

assert res == 2

assert b == 2
print("NamedExpressionScopeTest::test_named_expression_scope_20: ok")
"###);
    assert_output(&out, r###"NamedExpressionScopeTest::test_named_expression_scope_20: ok
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/named_expressions/named_expression_scope_test__test_named_expression_scope_21.py`.
#[test]
fn test_gen_behavior_std_libs_named_expressions_named_expression_scope_test__test_named_expression_scope_21() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "named_expressions"
# dimension = "behavior"
# case = "named_expression_scope_test__test_named_expression_scope_21"
# subject = "cpython.test_named_expressions.NamedExpressionScopeTest.test_named_expression_scope_21"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_named_expressions.py"
# status = "filled"
# ///
# Auto-ported from CPython 3.12 test_named_expressions.py::NamedExpressionScopeTest::test_named_expression_scope_21
"""Auto-ported test: NamedExpressionScopeTest::test_named_expression_scope_21 (CPython 3.12 oracle)."""


import unittest


GLOBAL_VAR = None


# --- test body ---
def spam(a, b):
    return a + b
res = spam((c := 2), b=1)

assert res == 3

assert c == 2
print("NamedExpressionScopeTest::test_named_expression_scope_21: ok")
"###);
    assert_output(&out, r###"NamedExpressionScopeTest::test_named_expression_scope_21: ok
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/named_expressions/named_expression_scope_test__test_named_expression_scope_22.py`.
#[test]
fn test_gen_behavior_std_libs_named_expressions_named_expression_scope_test__test_named_expression_scope_22() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "named_expressions"
# dimension = "behavior"
# case = "named_expression_scope_test__test_named_expression_scope_22"
# subject = "cpython.test_named_expressions.NamedExpressionScopeTest.test_named_expression_scope_22"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_named_expressions.py"
# status = "filled"
# ///
# Auto-ported from CPython 3.12 test_named_expressions.py::NamedExpressionScopeTest::test_named_expression_scope_22
"""Auto-ported test: NamedExpressionScopeTest::test_named_expression_scope_22 (CPython 3.12 oracle)."""


import unittest


GLOBAL_VAR = None


# --- test body ---
def spam(a, b):
    return a + b
res = spam((c := 2), b=1)

assert res == 3

assert c == 2
print("NamedExpressionScopeTest::test_named_expression_scope_22: ok")
"###);
    assert_output(&out, r###"NamedExpressionScopeTest::test_named_expression_scope_22: ok
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/named_expressions/named_expression_scope_test__test_named_expression_scope_23.py`.
#[test]
fn test_gen_behavior_std_libs_named_expressions_named_expression_scope_test__test_named_expression_scope_23() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "named_expressions"
# dimension = "behavior"
# case = "named_expression_scope_test__test_named_expression_scope_23"
# subject = "cpython.test_named_expressions.NamedExpressionScopeTest.test_named_expression_scope_23"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_named_expressions.py"
# status = "filled"
# ///
# Auto-ported from CPython 3.12 test_named_expressions.py::NamedExpressionScopeTest::test_named_expression_scope_23
"""Auto-ported test: NamedExpressionScopeTest::test_named_expression_scope_23 (CPython 3.12 oracle)."""


import unittest


GLOBAL_VAR = None


# --- test body ---
def spam(a, b):
    return a + b
res = spam(b=(c := 2), a=1)

assert res == 3

assert c == 2
print("NamedExpressionScopeTest::test_named_expression_scope_23: ok")
"###);
    assert_output(&out, r###"NamedExpressionScopeTest::test_named_expression_scope_23: ok
"###);
}
