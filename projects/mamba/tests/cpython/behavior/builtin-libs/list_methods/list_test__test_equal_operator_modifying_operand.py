# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "builtin-libs"
# lib = "list_methods"
# dimension = "behavior"
# case = "list_test__test_equal_operator_modifying_operand"
# subject = "cpython.test_list.ListTest.test_equal_operator_modifying_operand"
# kind = "semantic"
# xfail = "auto-ported CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_list.py"
# status = "filled"
# ///
# mamba-xfail: auto-ported CPython test; mamba promotion pending
# Auto-ported from CPython 3.12 test_list.py::ListTest::test_equal_operator_modifying_operand
"""Auto-ported test: ListTest::test_equal_operator_modifying_operand (CPython 3.12 oracle)."""


import sys
from test import list_tests
from test.support import cpython_only
import pickle
import unittest


# --- test body ---
type2test = list

class X:

    def __eq__(self, other):
        list2.clear()
        return NotImplemented

class Y:

    def __eq__(self, other):
        list1.clear()
        return NotImplemented

class Z:

    def __eq__(self, other):
        list3.clear()
        return NotImplemented
list1 = [X()]
list2 = [Y()]

assert list1 == list2
list3 = [Z()]
list4 = [1]

assert not list3 == list4
print("ListTest::test_equal_operator_modifying_operand: ok")
