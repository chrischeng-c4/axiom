# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "genericclass"
# dimension = "behavior"
# case = "test_class_getitem__test_class_getitem_with_builtins"
# subject = "cpython.test_genericclass.TestClassGetitem.test_class_getitem_with_builtins"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_genericclass.py"
# status = "filled"
# ///
# Auto-ported from CPython 3.12 test_genericclass.py::TestClassGetitem::test_class_getitem_with_builtins
"""Auto-ported test: TestClassGetitem::test_class_getitem_with_builtins (CPython 3.12 oracle)."""


import unittest
from test import support


# --- test body ---
class A(dict):
    called_with = None

    def __class_getitem__(cls, item):
        cls.called_with = item

class B(A):
    pass

assert B.called_with is None
B[int]

assert B.called_with is int
print("TestClassGetitem::test_class_getitem_with_builtins: ok")
