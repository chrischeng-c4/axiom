# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "builtin-libs"
# lib = "bool"
# dimension = "behavior"
# case = "bool_test__test_math"
# subject = "cpython.test.test_bool.BoolTest.test_math"
# kind = "semantic"
# xfail = "auto-ported CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_bool.py"
# status = "filled"
# ///
# mamba-xfail: auto-ported CPython test; mamba promotion pending
# Auto-ported from CPython 3.12 test_bool.py::BoolTest::test_math
"""Auto-ported test: BoolTest::test_math (CPython 3.12 oracle)."""


import warnings


def assert_warns_deprecation(fn):
    with warnings.catch_warnings(record=True) as caught:
        warnings.simplefilter("always", DeprecationWarning)
        fn()
    assert any(
        issubclass(item.category, DeprecationWarning) for item in caught
    ), caught


assert +False == 0
assert +False is not False
assert -False == 0
assert -False is not False
assert abs(False) == 0
assert abs(False) is not False
assert +True == 1
assert +True is not True
assert -True == -1
assert abs(True) == 1
assert abs(True) is not True


def check_invert_false_variable():
    false = False
    assert ~false == -1


def check_invert_false_eval():
    assert eval("~False") == -1


def check_invert_true_variable():
    true = True
    assert ~true == -2


def check_invert_true_eval():
    assert eval("~True") == -2


assert_warns_deprecation(check_invert_false_variable)
assert_warns_deprecation(check_invert_false_eval)
assert_warns_deprecation(check_invert_true_variable)
assert_warns_deprecation(check_invert_true_eval)

assert False + 2 == 2
assert True + 2 == 3
assert 2 + False == 2
assert 2 + True == 3

assert False + False == 0
assert False + False is not False
assert False + True == 1
assert False + True is not True
assert True + False == 1
assert True + False is not True
assert True + True == 2

assert True - True == 0
assert True - True is not False
assert False - False == 0
assert False - False is not False
assert True - False == 1
assert True - False is not True
assert False - True == -1

assert True * 1 == 1
assert False * 1 == 0
assert False * 1 is not False

assert True / 1 == 1
assert True / 1 is not True
assert False / 1 == 0
assert False / 1 is not False

assert True % 1 == 0
assert True % 1 is not False
assert True % 2 == 1
assert True % 2 is not True
assert False % 1 == 0
assert False % 1 is not False

for b in False, True:
    for i in 0, 1, 2:
        assert b**i == int(b) ** i
        assert b**i is not bool(int(b) ** i)

for a in False, True:
    for b in False, True:
        assert (a & b) is bool(int(a) & int(b))
        assert (a | b) is bool(int(a) | int(b))
        assert (a ^ b) is bool(int(a) ^ int(b))
        assert (a & int(b)) == (int(a) & int(b))
        assert (a & int(b)) is not bool(int(a) & int(b))
        assert (a | int(b)) == (int(a) | int(b))
        assert (a | int(b)) is not bool(int(a) | int(b))
        assert (a ^ int(b)) == (int(a) ^ int(b))
        assert (a ^ int(b)) is not bool(int(a) ^ int(b))
        assert (int(a) & b) == (int(a) & int(b))
        assert (int(a) & b) is not bool(int(a) & int(b))
        assert (int(a) | b) == (int(a) | int(b))
        assert (int(a) | b) is not bool(int(a) | int(b))
        assert (int(a) ^ b) == (int(a) ^ int(b))
        assert (int(a) ^ b) is not bool(int(a) ^ int(b))

assert (1 == 1) is True
assert (1 == 0) is False
assert (0 < 1) is True
assert (1 < 0) is False
assert (0 <= 0) is True
assert (1 <= 0) is False
assert (1 > 0) is True
assert (1 > 1) is False
assert (1 >= 1) is True
assert (0 >= 1) is False
assert (0 != 1) is True
assert (0 != 0) is False

x = [1]
assert (x is x) is True
assert (x is not x) is False
assert (1 in x) is True
assert (0 in x) is False
assert (1 not in x) is False
assert (0 not in x) is True

x = {1: 2}
assert (x is x) is True
assert (x is not x) is False
assert (1 in x) is True
assert (0 in x) is False
assert (1 not in x) is False
assert (0 not in x) is True

assert (not True) is False
assert (not False) is True

print("BoolTest::test_math: ok")
