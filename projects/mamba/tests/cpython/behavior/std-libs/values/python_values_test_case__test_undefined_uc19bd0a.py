# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "values"
# dimension = "behavior"
# case = "python_values_test_case__test_undefined_uc19bd0a"
# subject = "cpython.test_values.PythonValuesTestCase.test_undefined"
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
try:
    c_int.in_dll(pythonapi, 'Undefined_Symbol')
    raise AssertionError('assertRaises: no raise')
except ValueError:
    pass

print("PythonValuesTestCase::test_undefined: ok")
