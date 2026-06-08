# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "pointers"
# dimension = "behavior"
# case = "pointers_test_case__test_abstract_ucc9d80b"
# subject = "cpython.test_pointers.PointersTestCase.test_abstract"
# kind = "semantic"
# xfail = "auto-extracted CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_ctypes/test_pointers.py"
# status = "filled"
# ///
# mamba-xfail: auto-extracted CPython test; mamba promotion pending
from ctypes import *
import _ctypes_test
from ctypes import _Pointer
try:
    _Pointer.set_type(42)
    raise AssertionError('assertRaises: no raise')
except TypeError:
    pass

print("PointersTestCase::test_abstract: ok")
