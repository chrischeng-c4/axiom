# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "genericclass"
# dimension = "behavior"
# case = "test_class_getitem__test_class_getitem_metaclass"
# subject = "cpython.test_genericclass.TestClassGetitem.test_class_getitem_metaclass"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_genericclass.py"
# status = "filled"
# ///
# Auto-ported from CPython 3.12 test_genericclass.py::TestClassGetitem::test_class_getitem_metaclass
"""Auto-ported test: TestClassGetitem::test_class_getitem_metaclass (CPython 3.12 oracle)."""


import unittest
from test import support


# --- test body ---
class Meta(type):

    def __class_getitem__(cls, item):
        return f'{cls.__name__}[{item.__name__}]'

assert Meta[int] == 'Meta[int]'
print("TestClassGetitem::test_class_getitem_metaclass: ok")
