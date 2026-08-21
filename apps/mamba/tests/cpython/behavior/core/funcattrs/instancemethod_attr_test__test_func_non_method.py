# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "core"
# lib = "funcattrs"
# dimension = "behavior"
# case = "instancemethod_attr_test__test_func_non_method"
# subject = "cpython.test_funcattrs.InstancemethodAttrTest.test___func___non_method"
# kind = "semantic"
# xfail = "auto-ported CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_funcattrs.py"
# status = "filled"
# ///
# mamba-xfail: auto-ported CPython test; mamba promotion pending
# Auto-ported from CPython 3.12 test_funcattrs.py::InstancemethodAttrTest::test___func___non_method
"""Auto-ported test: InstancemethodAttrTest::test___func___non_method (CPython 3.12 oracle)."""


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
self_fi.id = types.MethodType(id, self_fi)

assert self_fi.id() == id(self_fi)
try:
    self_fi.id.unknown_attr
except AttributeError:
    pass
else:

    raise AssertionError('using unknown attributes should raise AttributeError')
cannot_set_attr(self_fi.id, 'unknown_attr', 2, AttributeError)
print("InstancemethodAttrTest::test___func___non_method: ok")
