# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "subclassinit"
# dimension = "behavior"
# case = "test__test_errors_changed_pep487"
# subject = "cpython.test_subclassinit.Test.test_errors_changed_pep487"
# kind = "semantic"
# xfail = "auto-ported CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_subclassinit.py"
# status = "filled"
# ///
# mamba-xfail: auto-ported CPython test; mamba promotion pending
# Auto-ported from CPython 3.12 test_subclassinit.py::Test::test_errors_changed_pep487
"""Auto-ported test: Test::test_errors_changed_pep487 (CPython 3.12 oracle)."""


import types
import unittest


# --- test body ---
class MyMeta(type):

    def __new__(cls, name, bases, namespace):
        return super().__new__(cls, name=name, bases=bases, dict=namespace)
try:

    class MyClass(metaclass=MyMeta):
        pass
    raise AssertionError('expected TypeError')
except TypeError:
    pass

class MyMeta(type):

    def __new__(cls, name, bases, namespace, otherarg):
        self = super().__new__(cls, name, bases, namespace)
        self.otherarg = otherarg
        return self

class MyClass2(metaclass=MyMeta, otherarg=1):
    pass

assert MyClass2.otherarg == 1
print("Test::test_errors_changed_pep487: ok")
