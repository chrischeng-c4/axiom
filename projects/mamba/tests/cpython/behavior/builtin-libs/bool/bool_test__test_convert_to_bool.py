# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "builtin-libs"
# lib = "bool"
# dimension = "behavior"
# case = "bool_test__test_convert_to_bool"
# subject = "cpython.test.test_bool.BoolTest.test_convert_to_bool"
# kind = "semantic"
# xfail = "auto-ported CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_bool.py"
# status = "filled"
# ///
# mamba-xfail: auto-ported CPython test; mamba promotion pending
# Auto-ported from CPython 3.12 test_bool.py::BoolTest::test_convert_to_bool
"""Auto-ported test: BoolTest::test_convert_to_bool (CPython 3.12 oracle)."""


def assert_raises(exc_type, fn, *args):
    try:
        fn(*args)
    except exc_type:
        return
    raise AssertionError(f"expected {exc_type.__name__}")


class Foo(object):
    def __bool__(self):
        return self


assert_raises(TypeError, bool, Foo())


class Bar(object):
    def __bool__(self):
        return "Yes"


assert_raises(TypeError, bool, Bar())


class Baz(int):
    def __bool__(self):
        return self


assert_raises(TypeError, bool, Baz())


class Spam(int):
    def __bool__(self):
        return 1


assert_raises(TypeError, bool, Spam())


class Eggs:
    def __len__(self):
        return -1


assert_raises(ValueError, bool, Eggs())

print("BoolTest::test_convert_to_bool: ok")
