# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "genericclass"
# dimension = "behavior"
# case = "test_class_getitem__test_class_getitem_format"
# subject = "cpython.test_genericclass.TestClassGetitem.test_class_getitem_format"
# kind = "semantic"
# xfail = "auto-ported CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_genericclass.py"
# status = "filled"
# ///
# mamba-xfail: auto-ported CPython test; mamba promotion pending
# Auto-ported from CPython 3.12 test_genericclass.py::TestClassGetitem::test_class_getitem_format
"""Auto-ported test: TestClassGetitem::test_class_getitem_format (CPython 3.12 oracle)."""


import unittest
from test import support


# --- test body ---
class C:

    def __class_getitem__(cls, item):
        return f'C[{item.__name__}]'

assert C[int] == 'C[int]'

assert C[C] == 'C[C]'
print("TestClassGetitem::test_class_getitem_format: ok")
