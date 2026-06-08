# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "values"
# dimension = "behavior"
# case = "values_test_case__test_undefined_uc1d8c41"
# subject = "cpython.test_values.ValuesTestCase.test_undefined"
# kind = "semantic"
# xfail = "auto-extracted CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_ctypes/test_values.py"
# status = "filled"
# ///
# mamba-xfail: auto-extracted CPython test; mamba promotion pending
import _imp
import importlib.util
import sys
from ctypes import *
import _ctypes_test
ctdll = CDLL(_ctypes_test.__file__)
try:
    c_int.in_dll(ctdll, 'Undefined_Symbol')
    raise AssertionError('assertRaises: no raise')
except ValueError:
    pass

print("ValuesTestCase::test_undefined: ok")
