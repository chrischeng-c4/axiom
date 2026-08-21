# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "slicing"
# dimension = "behavior"
# case = "slices_test_case__test_setslice_cint_uc880634"
# subject = "cpython.test_slicing.SlicesTestCase.test_setslice_cint"
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
a[32:47] = list(range(32, 47))
assert a[32:47] == list(range(32, 47))
a[32:47] = range(132, 147)
assert a[32:47] == list(range(132, 147))
a[46:31:-1] = range(232, 247)
assert a[32:47:1] == list(range(246, 231, -1))
a[32:47] = range(1132, 1147)
assert a[:] == b
a[32:47:7] = range(3)
b[32:47:7] = range(3)
assert a[:] == b
a[33::-3] = range(12)
b[33::-3] = range(12)
assert a[:] == b
from operator import setitem
try:
    setitem(a, slice(0, 5), 'abcde')
    raise AssertionError('assertRaises: no raise')
except TypeError:
    pass
try:
    setitem(a, slice(0, 5), ['a', 'b', 'c', 'd', 'e'])
    raise AssertionError('assertRaises: no raise')
except TypeError:
    pass
try:
    setitem(a, slice(0, 5), [1, 2, 3, 4, 3.14])
    raise AssertionError('assertRaises: no raise')
except TypeError:
    pass
try:
    setitem(a, slice(0, 5), range(32))
    raise AssertionError('assertRaises: no raise')
except ValueError:
    pass

print("SlicesTestCase::test_setslice_cint: ok")
