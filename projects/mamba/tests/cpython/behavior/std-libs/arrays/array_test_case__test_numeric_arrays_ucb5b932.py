# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "arrays"
# dimension = "behavior"
# case = "array_test_case__test_numeric_arrays_ucb5b932"
# subject = "cpython.test_arrays.ArrayTestCase.test_numeric_arrays"
# kind = "semantic"
# xfail = "auto-extracted CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_ctypes/test_arrays.py"
# status = "filled"
# ///
# mamba-xfail: auto-extracted CPython test; mamba promotion pending
import sys
from ctypes import *
alen = 5
numarray = ARRAY(c_int, alen)
na = numarray()
values = [na[i] for i in range(alen)]
assert values == [0] * alen
na = numarray(*[c_int()] * alen)
values = [na[i] for i in range(alen)]
assert values == [0] * alen
na = numarray(1, 2, 3, 4, 5)
values = [i for i in na]
assert values == [1, 2, 3, 4, 5]
na = numarray(*map(c_int, (1, 2, 3, 4, 5)))
values = [i for i in na]
assert values == [1, 2, 3, 4, 5]

print("ArrayTestCase::test_numeric_arrays: ok")
