# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "builtin-libs"
# lib = "list_methods"
# dimension = "behavior"
# case = "list_test__test_keywords_in_subclass"
# subject = "cpython.test_list.ListTest.test_keywords_in_subclass"
# kind = "semantic"
# xfail = "auto-ported CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_list.py"
# status = "filled"
# ///
# mamba-xfail: auto-ported CPython test; mamba promotion pending
# Auto-ported from CPython 3.12 test_list.py::ListTest::test_keywords_in_subclass
"""Auto-ported test: ListTest::test_keywords_in_subclass (CPython 3.12 oracle)."""


import sys
from test import list_tests
from test.support import cpython_only
import pickle
import unittest


# --- test body ---
type2test = list

class subclass(list):
    pass
u = subclass([1, 2])

assert type(u) is subclass

assert list(u) == [1, 2]
try:
    subclass(sequence=())
    raise AssertionError('expected TypeError')
except TypeError:
    pass

class subclass_with_init(list):

    def __init__(self, seq, newarg=None):
        super().__init__(seq)
        self.newarg = newarg
u = subclass_with_init([1, 2], newarg=3)

assert type(u) is subclass_with_init

assert list(u) == [1, 2]

assert u.newarg == 3

class subclass_with_new(list):

    def __new__(cls, seq, newarg=None):
        self = super().__new__(cls, seq)
        self.newarg = newarg
        return self
u = subclass_with_new([1, 2], newarg=3)

assert type(u) is subclass_with_new

assert list(u) == [1, 2]

assert u.newarg == 3
print("ListTest::test_keywords_in_subclass: ok")
