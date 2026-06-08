# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "keeprefs"
# dimension = "behavior"
# case = "pointer_test_case__test_p_cint_uc6c4890"
# subject = "cpython.test_keeprefs.PointerTestCase.test_p_cint"
# kind = "semantic"
# xfail = "auto-extracted CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_ctypes/test_keeprefs.py"
# status = "filled"
# ///
# mamba-xfail: auto-extracted CPython test; mamba promotion pending
from ctypes import *
i = c_int(42)
x = pointer(i)
assert x._objects == {'1': i}

print("PointerTestCase::test_p_cint: ok")
