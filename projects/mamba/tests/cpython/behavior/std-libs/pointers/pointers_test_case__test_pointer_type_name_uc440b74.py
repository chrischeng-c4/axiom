# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "pointers"
# dimension = "behavior"
# case = "pointers_test_case__test_pointer_type_name_uc440b74"
# subject = "cpython.test_pointers.PointersTestCase.test_pointer_type_name"
# kind = "semantic"
# xfail = "auto-extracted CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_ctypes/test_pointers.py"
# status = "filled"
# ///
# mamba-xfail: auto-extracted CPython test; mamba promotion pending
from ctypes import *
import _ctypes_test
LargeNamedType = type('T' * 2 ** 25, (Structure,), {})
assert POINTER(LargeNamedType)
from ctypes import _pointer_type_cache
del _pointer_type_cache[LargeNamedType]

print("PointersTestCase::test_pointer_type_name: ok")
