# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "core"
# lib = "funcattrs"
# dimension = "behavior"
# case = "function_properties_test__test_duplicate_function_equality"
# subject = "cpython.test_funcattrs.FunctionPropertiesTest.test_duplicate_function_equality"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_funcattrs.py"
# status = "filled"
# ///
# Auto-ported from CPython 3.12 test_funcattrs.py::FunctionPropertiesTest::test_duplicate_function_equality
"""Auto-ported test: FunctionPropertiesTest::test_duplicate_function_equality (CPython 3.12 oracle)."""


import textwrap
import types
import typing
import unittest


def global_function():

    def inner_function():

        class LocalClass:
            pass
        global inner_global_function

        def inner_global_function():

            def inner_function2():
                pass
            return inner_function2
        return LocalClass
    return lambda: inner_function

class FuncAttrsTest(unittest.TestCase):

    def setUp(self):

        class F:

            def a(self):
                pass

        def b():
            return 3
        self.fi = F()
        self.F = F
        self.b = b

    def cannot_set_attr(self, obj, name, value, exceptions):
        try:
            setattr(obj, name, value)
        except exceptions:
            pass
        else:
            self.fail("shouldn't be able to set %s to %r" % (name, value))
        try:
            delattr(obj, name)
        except exceptions:
            pass
        else:
            self.fail("shouldn't be able to del %s" % name)

def cell(value):
    """Create a cell containing the given value."""

    def f():
        print(a)
    a = value
    return f.__closure__[0]

def empty_cell(empty=True):
    """Create an empty cell."""

    def f():
        print(a)
    if not empty:
        a = 1729
    return f.__closure__[0]


# --- test body ---
class F:

    def a(self):
        pass

def b():
    return 3
self_fi = F()
self_F = F
self_b = b

def duplicate():
    """my docstring"""
    return 3

assert self_b != duplicate
print("FunctionPropertiesTest::test_duplicate_function_equality: ok")
