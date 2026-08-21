# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "isinstance"
# dimension = "behavior"
# case = "test_is_instance_is_subclass__test_isinstance_normal"
# subject = "cpython.test_isinstance.TestIsInstanceIsSubclass.test_isinstance_normal"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_isinstance.py"
# status = "filled"
# ///
# Auto-ported from CPython 3.12 test_isinstance.py::TestIsInstanceIsSubclass::test_isinstance_normal
"""Auto-ported test: TestIsInstanceIsSubclass::test_isinstance_normal (CPython 3.12 oracle)."""


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

assert True == isinstance(Super(), Super)

assert False == isinstance(Super(), Child)

assert False == isinstance(Super(), AbstractSuper)

assert False == isinstance(Super(), AbstractChild)

assert True == isinstance(Child(), Super)

assert False == isinstance(Child(), AbstractSuper)
print("TestIsInstanceIsSubclass::test_isinstance_normal: ok")
