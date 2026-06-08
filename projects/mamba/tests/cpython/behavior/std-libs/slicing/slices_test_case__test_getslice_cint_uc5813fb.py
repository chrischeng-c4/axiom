# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "slicing"
# dimension = "behavior"
# case = "slices_test_case__test_getslice_cint_uc5813fb"
# subject = "cpython.test_slicing.SlicesTestCase.test_getslice_cint"
# kind = "semantic"
# xfail = "auto-extracted CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_ctypes/test_slicing.py"
# status = "filled"
# ///
# mamba-xfail: auto-extracted CPython test; mamba promotion pending
from ctypes import *
import _ctypes_test
a = (c_int * 100)(*range(1100, 1200))
b = list(range(1100, 1200))
assert a[0:2] == b[0:2]
assert a[0:2] == b[0:2]
assert len(a) == len(b)
assert a[5:7] == b[5:7]
assert a[5:7] == b[5:7]
assert a[-1] == b[-1]
assert a[:] == b[:]
assert a[:] == b[:]
assert a[10::-1] == b[10::-1]
assert a[30:20:-1] == b[30:20:-1]
assert a[:12:6] == b[:12:6]
assert a[2:6:4] == b[2:6:4]
a[0:5] = range(5, 10)
assert a[0:5] == list(range(5, 10))
assert a[0:5] == list(range(5, 10))
assert a[4::-1] == list(range(9, 4, -1))

print("SlicesTestCase::test_getslice_cint: ok")
