# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "isinstance"
# dimension = "behavior"
# case = "test_is_instance_is_subclass__test_subclass_with_union"
# subject = "cpython.test_isinstance.TestIsInstanceIsSubclass.test_subclass_with_union"
# kind = "semantic"
# xfail = "auto-ported CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_isinstance.py"
# status = "filled"
# ///
# mamba-xfail: auto-ported CPython test; mamba promotion pending
# Auto-ported from CPython 3.12 test_isinstance.py::TestIsInstanceIsSubclass::test_subclass_with_union
"""Auto-ported test: TestIsInstanceIsSubclass::test_subclass_with_union (CPython 3.12 oracle)."""


import unittest
import typing
from test import support


class AbstractClass(object):

    def __init__(self, bases):
        self.bases = bases

    def getbases(self):
        return self.bases
    __bases__ = property(getbases)

    def __call__(self):
        return AbstractInstance(self)

class AbstractInstance(object):

    def __init__(self, klass):
        self.klass = klass

    def getclass(self):
        return self.klass
    __class__ = property(getclass)

AbstractSuper = AbstractClass(bases=())

AbstractChild = AbstractClass(bases=(AbstractSuper,))

class Super:
    pass

class Child(Super):
    pass

def blowstack(fxn, arg, compare_to):
    tuple_arg = (compare_to,)
    for cnt in range(support.C_RECURSION_LIMIT * 2):
        tuple_arg = (tuple_arg,)
        fxn(arg, tuple_arg)


# --- test body ---

assert issubclass(int, int | float | int)

assert issubclass(str, str | Child | str)

assert not issubclass(dict, float | str)

assert not issubclass(object, float | str)
try:
    issubclass(2, Child | Super)
    raise AssertionError('expected TypeError')
except TypeError:
    pass
try:
    issubclass(int, list[int] | Child)
    raise AssertionError('expected TypeError')
except TypeError:
    pass
print("TestIsInstanceIsSubclass::test_subclass_with_union: ok")
