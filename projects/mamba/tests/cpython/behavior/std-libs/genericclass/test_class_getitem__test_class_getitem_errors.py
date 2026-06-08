# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "genericclass"
# dimension = "behavior"
# case = "test_class_getitem__test_class_getitem_errors"
# subject = "cpython.test_genericclass.TestClassGetitem.test_class_getitem_errors"
# kind = "semantic"
# xfail = "auto-ported CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_genericclass.py"
# status = "filled"
# ///
# mamba-xfail: auto-ported CPython test; mamba promotion pending
# Auto-ported from CPython 3.12 test_genericclass.py::TestClassGetitem::test_class_getitem_errors
"""Auto-ported test: TestClassGetitem::test_class_getitem_errors (CPython 3.12 oracle)."""


import unittest
from test import support


# --- test body ---
class C_too_few:

    def __class_getitem__(cls):
        return None
try:
    C_too_few[int]
    raise AssertionError('expected TypeError')
except TypeError:
    pass

class C_too_many:

    def __class_getitem__(cls, one, two):
        return None
try:
    C_too_many[int]
    raise AssertionError('expected TypeError')
except TypeError:
    pass
print("TestClassGetitem::test_class_getitem_errors: ok")
