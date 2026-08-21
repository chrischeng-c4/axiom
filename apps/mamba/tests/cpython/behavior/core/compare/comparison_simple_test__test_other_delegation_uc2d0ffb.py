# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "core"
# lib = "compare"
# dimension = "behavior"
# case = "comparison_simple_test__test_other_delegation_uc2d0ffb"
# subject = "cpython.test_compare.ComparisonSimpleTest.test_other_delegation"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_compare.py"
# status = "filled"
# ///
def boom_ne(*args):
    raise AssertionError("__ne__ called")


def boom_eq(*args):
    raise AssertionError("__eq__ called")


def boom_lt(*args):
    raise AssertionError("__lt__ called")


def boom_le(*args):
    raise AssertionError("__le__ called")


def boom_gt(*args):
    raise AssertionError("__gt__ called")


def boom_ge(*args):
    raise AssertionError("__ge__ called")


def expect_type_error(func):
    try:
        func()
    except TypeError:
        return
    raise AssertionError("missing rich comparison method did not raise TypeError")


class MissingEq:
    __ne__ = boom_ne
    __lt__ = boom_lt
    __le__ = boom_le
    __gt__ = boom_gt
    __ge__ = boom_ge


assert (MissingEq() == object()) is False


class MissingLt:
    __ne__ = boom_ne
    __eq__ = boom_eq
    __le__ = boom_le
    __gt__ = boom_gt
    __ge__ = boom_ge


expect_type_error(lambda: MissingLt() < object())


class MissingLe:
    __ne__ = boom_ne
    __eq__ = boom_eq
    __lt__ = boom_lt
    __gt__ = boom_gt
    __ge__ = boom_ge


expect_type_error(lambda: MissingLe() <= object())


class MissingGt:
    __ne__ = boom_ne
    __eq__ = boom_eq
    __lt__ = boom_lt
    __le__ = boom_le
    __ge__ = boom_ge


expect_type_error(lambda: MissingGt() > object())


class MissingGe:
    __ne__ = boom_ne
    __eq__ = boom_eq
    __lt__ = boom_lt
    __le__ = boom_le
    __gt__ = boom_gt


expect_type_error(lambda: MissingGe() >= object())

print("ComparisonSimpleTest::test_other_delegation: ok")
