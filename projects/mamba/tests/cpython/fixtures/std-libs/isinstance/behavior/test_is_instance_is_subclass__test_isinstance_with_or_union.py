# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "isinstance"
# dimension = "behavior"
# case = "test_is_instance_is_subclass__test_isinstance_with_or_union"
# subject = "cpython.test_isinstance.TestIsInstanceIsSubclass.test_isinstance_with_or_union"
# kind = "semantic"
# xfail = "auto-ported CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_isinstance.py"
# status = "filled"
# ///
# mamba-xfail: auto-ported CPython test; mamba promotion pending
# Auto-ported from CPython 3.12 test_isinstance.py::TestIsInstanceIsSubclass::test_isinstance_with_or_union
"""Auto-ported test: TestIsInstanceIsSubclass::test_isinstance_with_or_union (CPython 3.12 oracle)."""


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

assert isinstance(Super(), Super | int)

assert not isinstance(None, str | int)

assert isinstance(3, str | int)

assert isinstance('', str | int)

assert isinstance([], typing.List | typing.Tuple)

assert isinstance(2, typing.List | int)

assert not isinstance(2, typing.List | typing.Tuple)

assert isinstance(None, int | None)

assert not isinstance(3.14, int | str)
try:
    isinstance(2, list[int])
    raise AssertionError('expected TypeError')
except TypeError:
    pass
try:
    isinstance(2, list[int] | int)
    raise AssertionError('expected TypeError')
except TypeError:
    pass
try:
    isinstance(2, float | str | list[int] | int)
    raise AssertionError('expected TypeError')
except TypeError:
    pass
print("TestIsInstanceIsSubclass::test_isinstance_with_or_union: ok")
