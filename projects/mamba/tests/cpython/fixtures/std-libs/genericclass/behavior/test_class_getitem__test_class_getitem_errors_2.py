# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "genericclass"
# dimension = "behavior"
# case = "test_class_getitem__test_class_getitem_errors_2"
# subject = "cpython.test_genericclass.TestClassGetitem.test_class_getitem_errors_2"
# kind = "semantic"
# xfail = "auto-ported CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_genericclass.py"
# status = "filled"
# ///
# mamba-xfail: auto-ported CPython test; mamba promotion pending
# Auto-ported from CPython 3.12 test_genericclass.py::TestClassGetitem::test_class_getitem_errors_2
"""Auto-ported test: TestClassGetitem::test_class_getitem_errors_2 (CPython 3.12 oracle)."""


import unittest
from test import support


# --- test body ---
class C:

    def __class_getitem__(cls, item):
        return None
try:
    C()[int]
    raise AssertionError('expected TypeError')
except TypeError:
    pass

class E:
    ...
e = E()
e.__class_getitem__ = lambda cls, item: 'This will not work'
try:
    e[int]
    raise AssertionError('expected TypeError')
except TypeError:
    pass

class C_not_callable:
    __class_getitem__ = 'Surprise!'
try:
    C_not_callable[int]
    raise AssertionError('expected TypeError')
except TypeError:
    pass

class C_is_none(tuple):
    __class_getitem__ = None
try:
    C_is_none[int]
    raise AssertionError('expected TypeError')
except TypeError as _aR_e:
    import re as _re_aR
    assert _re_aR.search('C_is_none', str(_aR_e))
print("TestClassGetitem::test_class_getitem_errors_2: ok")
