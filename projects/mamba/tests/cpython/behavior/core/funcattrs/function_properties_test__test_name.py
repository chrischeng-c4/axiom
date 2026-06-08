# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "core"
# lib = "funcattrs"
# dimension = "behavior"
# case = "function_properties_test__test_name"
# subject = "cpython.test_funcattrs.FunctionPropertiesTest.test___name__"
# kind = "semantic"
# xfail = "auto-ported CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_funcattrs.py"
# status = "filled"
# ///
# mamba-xfail: auto-ported CPython test; mamba promotion pending
# Auto-ported from CPython 3.12 test_funcattrs.py::FunctionPropertiesTest::test___name__
"""Auto-ported test: FunctionPropertiesTest::test___name__ (CPython 3.12 oracle)."""


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
def cannot_set_attr(obj, name, value, exceptions):
    try:
        setattr(obj, name, value)
    except exceptions:
        pass
    else:

        raise AssertionError("shouldn't be able to set %s to %r" % (name, value))
    try:
        delattr(obj, name)
    except exceptions:
        pass
    else:

        raise AssertionError("shouldn't be able to del %s" % name)

class F:

    def a(self):
        pass

def b():
    return 3
self_fi = F()
self_F = F
self_b = b

assert self_b.__name__ == 'b'
self_b.__name__ = 'c'

assert self_b.__name__ == 'c'
self_b.__name__ = 'd'

assert self_b.__name__ == 'd'
cannot_set_attr(self_b, '__name__', 7, TypeError)
s = 'def f(): pass\nf.__name__'
exec(s, {'__builtins__': {}})

assert self_fi.a.__name__ == 'a'
cannot_set_attr(self_fi.a, '__name__', 'a', AttributeError)
print("FunctionPropertiesTest::test___name__: ok")
