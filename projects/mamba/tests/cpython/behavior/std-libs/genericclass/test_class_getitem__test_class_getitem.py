# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "genericclass"
# dimension = "behavior"
# case = "test_class_getitem__test_class_getitem"
# subject = "cpython.test_genericclass.TestClassGetitem.test_class_getitem"
# kind = "semantic"
# xfail = "auto-ported CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_genericclass.py"
# status = "filled"
# ///
# mamba-xfail: auto-ported CPython test; mamba promotion pending
# Auto-ported from CPython 3.12 test_genericclass.py::TestClassGetitem::test_class_getitem
"""Auto-ported test: TestClassGetitem::test_class_getitem (CPython 3.12 oracle)."""


import unittest
from test import support


# --- test body ---
getitem_args = []

class C:

    def __class_getitem__(*args, **kwargs):
        getitem_args.extend([args, kwargs])
        return None
C[int, str]

assert getitem_args[0] == (C, (int, str))

assert getitem_args[1] == {}
print("TestClassGetitem::test_class_getitem: ok")
