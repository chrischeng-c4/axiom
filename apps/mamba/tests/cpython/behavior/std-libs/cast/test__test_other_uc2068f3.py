# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "cast"
# dimension = "behavior"
# case = "test__test_other_uc2068f3"
# subject = "cpython.test_cast.Test.test_other"
# kind = "semantic"
# xfail = "auto-extracted CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_ctypes/test_cast.py"
# status = "filled"
# ///
# mamba-xfail: auto-extracted CPython test; mamba promotion pending
from ctypes import *
import sys
p = cast((c_int * 4)(1, 2, 3, 4), POINTER(c_int))
assert p[:4] == [1, 2, 3, 4]
assert p[:4] == [1, 2, 3, 4]
assert p[3:-1:-1] == [4, 3, 2, 1]
assert p[:4:3] == [1, 4]
c_int()
assert p[:4] == [1, 2, 3, 4]
assert p[:4] == [1, 2, 3, 4]
assert p[3:-1:-1] == [4, 3, 2, 1]
assert p[:4:3] == [1, 4]
p[2] = 96
assert p[:4] == [1, 2, 96, 4]
assert p[:4] == [1, 2, 96, 4]
assert p[3:-1:-1] == [4, 96, 2, 1]
assert p[:4:3] == [1, 4]
c_int()
assert p[:4] == [1, 2, 96, 4]
assert p[:4] == [1, 2, 96, 4]
assert p[3:-1:-1] == [4, 96, 2, 1]
assert p[:4:3] == [1, 4]

print("Test::test_other: ok")
