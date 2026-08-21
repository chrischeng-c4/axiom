# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "internals"
# dimension = "behavior"
# case = "objects_test_case__test_ints_ucb66a6c"
# subject = "cpython.test_internals.ObjectsTestCase.test_ints"
# kind = "semantic"
# xfail = "auto-extracted CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_ctypes/test_internals.py"
# status = "filled"
# ///
# mamba-xfail: auto-extracted CPython test; mamba promotion pending
from ctypes import *
from sys import getrefcount as grc

def assertSame(a, b):
    assert id(a) == id(b)
i = 42000123
refcnt = grc(i)
ci = c_int(i)
assert refcnt == grc(i)
assert ci._objects == None

print("ObjectsTestCase::test_ints: ok")
