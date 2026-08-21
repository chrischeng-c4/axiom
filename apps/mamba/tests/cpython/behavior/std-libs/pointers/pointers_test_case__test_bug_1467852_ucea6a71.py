# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "pointers"
# dimension = "behavior"
# case = "pointers_test_case__test_bug_1467852_ucea6a71"
# subject = "cpython.test_pointers.PointersTestCase.test_bug_1467852"
# kind = "semantic"
# xfail = "auto-extracted CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_ctypes/test_pointers.py"
# status = "filled"
# ///
# mamba-xfail: auto-extracted CPython test; mamba promotion pending
from ctypes import *
import _ctypes_test
x = c_int(5)
dummy = []
for i in range(32000):
    dummy.append(c_int(i))
y = c_int(6)
p = pointer(x)
pp = pointer(p)
q = pointer(y)
pp[0] = q
assert p[0] == 6

print("PointersTestCase::test_bug_1467852: ok")
