# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "numbers"
# dimension = "behavior"
# case = "number_test_case__test_init_ucc7aa69"
# subject = "cpython.test_numbers.NumberTestCase.test_init"
# kind = "semantic"
# xfail = "auto-extracted CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_ctypes/test_numbers.py"
# status = "filled"
# ///
# mamba-xfail: auto-extracted CPython test; mamba promotion pending
from ctypes import *
import struct
try:
    c_int(c_long(42))
    raise AssertionError('assertRaises: no raise')
except TypeError:
    pass

print("NumberTestCase::test_init: ok")
