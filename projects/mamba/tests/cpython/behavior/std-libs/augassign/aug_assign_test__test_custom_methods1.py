# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "augassign"
# dimension = "behavior"
# case = "aug_assign_test__test_custom_methods1"
# subject = "cpython.test_augassign.AugAssignTest.testCustomMethods1"
# kind = "semantic"
# xfail = "auto-ported CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_augassign.py"
# status = "filled"
# ///
# mamba-xfail: auto-ported CPython test; mamba promotion pending
# Auto-ported from CPython 3.12 test_augassign.py::AugAssignTest::testCustomMethods1
"""Auto-ported test: AugAssignTest::testCustomMethods1 (CPython 3.12 oracle)."""


import unittest


# --- test body ---
class aug_test:

    def __init__(self, value):
        self.val = value

    def __radd__(self, val):
        return self.val + val

    def __add__(self, val):
        return aug_test(self.val + val)

class aug_test2(aug_test):

    def __iadd__(self, val):
        self.val = self.val + val
        return self

class aug_test3(aug_test):

    def __iadd__(self, val):
        return aug_test3(self.val + val)

class aug_test4(aug_test3):
    """Blocks inheritance, and fallback to __add__"""
    __iadd__ = None
x = aug_test(1)
y = x
x += 10

assert isinstance(x, aug_test)

assert y is not x

assert x.val == 11
x = aug_test2(2)
y = x
x += 10

assert y is x

assert x.val == 12
x = aug_test3(3)
y = x
x += 10

assert isinstance(x, aug_test3)

assert y is not x

assert x.val == 13
x = aug_test4(4)
try:
    x += 10
    raise AssertionError('expected TypeError')
except TypeError:
    pass
print("AugAssignTest::testCustomMethods1: ok")
