# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "pointers"
# dimension = "behavior"
# case = "pointers_test_case__test_basic_uc435d98"
# subject = "cpython.test_pointers.PointersTestCase.test_basic"
# kind = "semantic"
# xfail = "auto-extracted CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_ctypes/test_pointers.py"
# status = "filled"
# ///
# mamba-xfail: auto-extracted CPython test; mamba promotion pending
from ctypes import *
import _ctypes_test
p = pointer(c_int(42))
try:
    len(p)
    raise AssertionError('assertRaises: no raise')
except TypeError:
    pass
assert p[0] == 42
assert p[0:1] == [42]
assert p.contents.value == 42

print("PointersTestCase::test_basic: ok")
