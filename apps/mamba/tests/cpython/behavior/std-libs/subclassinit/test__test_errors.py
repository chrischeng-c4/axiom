# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "subclassinit"
# dimension = "behavior"
# case = "test__test_errors"
# subject = "cpython.test_subclassinit.Test.test_errors"
# kind = "semantic"
# xfail = "auto-ported CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_subclassinit.py"
# status = "filled"
# ///
# mamba-xfail: auto-ported CPython test; mamba promotion pending
# Auto-ported from CPython 3.12 test_subclassinit.py::Test::test_errors
"""Auto-ported test: Test::test_errors (CPython 3.12 oracle)."""


import types
import unittest


# --- test body ---
class MyMeta(type):
    pass
try:

    class MyClass(metaclass=MyMeta, otherarg=1):
        pass
    raise AssertionError('expected TypeError')
except TypeError:
    pass
try:
    types.new_class('MyClass', (object,), dict(metaclass=MyMeta, otherarg=1))
    raise AssertionError('expected TypeError')
except TypeError:
    pass
types.prepare_class('MyClass', (object,), dict(metaclass=MyMeta, otherarg=1))

class MyMeta(type):

    def __init__(self, name, bases, namespace, otherarg):
        super().__init__(name, bases, namespace)
try:

    class MyClass2(metaclass=MyMeta, otherarg=1):
        pass
    raise AssertionError('expected TypeError')
except TypeError:
    pass

class MyMeta(type):

    def __new__(cls, name, bases, namespace, otherarg):
        return super().__new__(cls, name, bases, namespace)

    def __init__(self, name, bases, namespace, otherarg):
        super().__init__(name, bases, namespace)
        self.otherarg = otherarg

class MyClass3(metaclass=MyMeta, otherarg=1):
    pass

assert MyClass3.otherarg == 1
print("Test::test_errors: ok")
