# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "genericclass"
# dimension = "behavior"
# case = "test_class_getitem__test_class_getitem_classmethod"
# subject = "cpython.test_genericclass.TestClassGetitem.test_class_getitem_classmethod"
# kind = "semantic"
# xfail = "auto-ported CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_genericclass.py"
# status = "filled"
# ///
# mamba-xfail: auto-ported CPython test; mamba promotion pending
# Auto-ported from CPython 3.12 test_genericclass.py::TestClassGetitem::test_class_getitem_classmethod
"""Auto-ported test: TestClassGetitem::test_class_getitem_classmethod (CPython 3.12 oracle)."""


import unittest
from test import support


# --- test body ---
class C:

    @classmethod
    def __class_getitem__(cls, item):
        return f'{cls.__name__}[{item.__name__}]'

class D(C):
    ...

assert D[int] == 'D[int]'

assert D[D] == 'D[D]'
print("TestClassGetitem::test_class_getitem_classmethod: ok")
