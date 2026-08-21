# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "core"
# lib = "funcattrs"
# dimension = "behavior"
# case = "builtin_function_properties_test__test_builtin_qualname"
# subject = "cpython.test_funcattrs.BuiltinFunctionPropertiesTest.test_builtin__qualname__"
# kind = "semantic"
# xfail = "auto-ported CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_funcattrs.py"
# status = "filled"
# ///
# mamba-xfail: auto-ported CPython test; mamba promotion pending
# Auto-ported from CPython 3.12 test_funcattrs.py::BuiltinFunctionPropertiesTest::test_builtin__qualname__
"""Auto-ported test: BuiltinFunctionPropertiesTest::test_builtin__qualname__ (CPython 3.12 oracle)."""


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
import time

assert len.__qualname__ == 'len'

assert time.time.__qualname__ == 'time'

assert dict.fromkeys.__qualname__ == 'dict.fromkeys'

assert float.__getformat__.__qualname__ == 'float.__getformat__'

assert str.maketrans.__qualname__ == 'str.maketrans'

assert bytes.maketrans.__qualname__ == 'bytes.maketrans'

assert [1, 2, 3].append.__qualname__ == 'list.append'

assert {'foo': 'bar'}.pop.__qualname__ == 'dict.pop'
print("BuiltinFunctionPropertiesTest::test_builtin__qualname__: ok")
