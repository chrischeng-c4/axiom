# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "values"
# dimension = "behavior"
# case = "values_test_case__test_an_integer_uc13dcae"
# subject = "cpython.test_values.ValuesTestCase.test_an_integer"
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
an_integer = c_int.in_dll(ctdll, 'an_integer')
x = an_integer.value
assert x == ctdll.get_an_integer()
an_integer.value *= 2
assert x * 2 == ctdll.get_an_integer()
an_integer.value = x
assert x == ctdll.get_an_integer()

print("ValuesTestCase::test_an_integer: ok")
