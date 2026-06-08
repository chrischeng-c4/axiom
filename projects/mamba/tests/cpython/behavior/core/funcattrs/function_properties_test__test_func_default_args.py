# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "core"
# lib = "funcattrs"
# dimension = "behavior"
# case = "function_properties_test__test_func_default_args"
# subject = "cpython.test_funcattrs.FunctionPropertiesTest.test_func_default_args"
# kind = "semantic"
# xfail = "auto-ported CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_funcattrs.py"
# status = "filled"
# ///
# mamba-xfail: auto-ported CPython test; mamba promotion pending
# Auto-ported from CPython 3.12 test_funcattrs.py::FunctionPropertiesTest::test_func_default_args
"""Auto-ported test: FunctionPropertiesTest::test_func_default_args (CPython 3.12 oracle)."""


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

def first_func(a, b):
    return a + b

def second_func(a=1, b=2):
    return a + b

assert first_func.__defaults__ == None

assert second_func.__defaults__ == (1, 2)
first_func.__defaults__ = (1, 2)

assert first_func.__defaults__ == (1, 2)

assert first_func() == 3

assert first_func(3) == 5

assert first_func(3, 5) == 8
del second_func.__defaults__

assert second_func.__defaults__ == None
try:
    second_func()
except TypeError:
    pass
else:

    raise AssertionError('__defaults__ does not update; deleting it does not remove requirement')
print("FunctionPropertiesTest::test_func_default_args: ok")
