# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "builtin-libs"
# lib = "list_methods"
# dimension = "behavior"
# case = "list_test__test_basic"
# subject = "cpython.test_list.ListTest.test_basic"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_list.py"
# status = "filled"
# ///
# Auto-ported from CPython 3.12 test_list.py::ListTest::test_basic
"""Auto-ported test: ListTest::test_basic (CPython 3.12 oracle)."""


import sys
from test import list_tests
from test.support import cpython_only
import pickle
import unittest


# --- test body ---
type2test = list

assert list([]) == []
l0_3 = [0, 1, 2, 3]
l0_3_bis = list(l0_3)

assert l0_3 == l0_3_bis

assert l0_3 is not l0_3_bis

assert list(()) == []

assert list((0, 1, 2, 3)) == [0, 1, 2, 3]

assert list('') == []

assert list('spam') == ['s', 'p', 'a', 'm']

assert list((x for x in range(10) if x % 2)) == [1, 3, 5, 7, 9]
if sys.maxsize == 2147483647:

    try:
        list(range(sys.maxsize // 2))
        raise AssertionError('expected MemoryError')
    except MemoryError:
        pass
x = []
x.extend((-y for y in x))

assert x == []
print("ListTest::test_basic: ok")
