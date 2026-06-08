# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "isinstance"
# dimension = "behavior"
# case = "test_is_instance_is_subclass__test_subclass_normal"
# subject = "cpython.test_isinstance.TestIsInstanceIsSubclass.test_subclass_normal"
# kind = "semantic"
# xfail = "auto-ported CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_isinstance.py"
# status = "filled"
# ///
# mamba-xfail: auto-ported CPython test; mamba promotion pending
# Auto-ported from CPython 3.12 test_isinstance.py::TestIsInstanceIsSubclass::test_subclass_normal
"""Auto-ported test: TestIsInstanceIsSubclass::test_subclass_normal (CPython 3.12 oracle)."""


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

assert True == issubclass(Super, Super)

assert False == issubclass(Super, AbstractSuper)

assert False == issubclass(Super, Child)

assert True == issubclass(Child, Child)

assert True == issubclass(Child, Super)

assert False == issubclass(Child, AbstractSuper)

assert issubclass(typing.List, typing.List | typing.Tuple)

assert not issubclass(int, typing.List | typing.Tuple)
print("TestIsInstanceIsSubclass::test_subclass_normal: ok")
