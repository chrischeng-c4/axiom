# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "funcptr"
# dimension = "behavior"
# case = "c_func_ptr_test_case__test_abstract_uc9fecd2"
# subject = "cpython.test_funcptr.CFuncPtrTestCase.test_abstract"
# kind = "semantic"
# xfail = "auto-extracted CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_ctypes/test_funcptr.py"
# status = "filled"
# ///
# mamba-xfail: auto-extracted CPython test; mamba promotion pending
from ctypes import *
import _ctypes_test
from ctypes import _CFuncPtr
try:
    _CFuncPtr(13, 'name', 42, 'iid')
    raise AssertionError('assertRaises: no raise')
except TypeError:
    pass

print("CFuncPtrTestCase::test_abstract: ok")
