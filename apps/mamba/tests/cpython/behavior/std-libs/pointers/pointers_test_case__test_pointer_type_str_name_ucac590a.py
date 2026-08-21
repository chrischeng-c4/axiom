# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "pointers"
# dimension = "behavior"
# case = "pointers_test_case__test_pointer_type_str_name_ucac590a"
# subject = "cpython.test_pointers.PointersTestCase.test_pointer_type_str_name"
# kind = "semantic"
# xfail = "auto-extracted CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_ctypes/test_pointers.py"
# status = "filled"
# ///
# mamba-xfail: auto-extracted CPython test; mamba promotion pending
from ctypes import *
import _ctypes_test
large_string = 'T' * 2 ** 25
P = POINTER(large_string)
assert P
from ctypes import _pointer_type_cache
del _pointer_type_cache[id(P)]

print("PointersTestCase::test_pointer_type_str_name: ok")
