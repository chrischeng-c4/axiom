# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "internals"
# dimension = "behavior"
# case = "objects_test_case__test_c_char_p_uc50d687"
# subject = "cpython.test_internals.ObjectsTestCase.test_c_char_p"
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
s = b'Hello, World'
refcnt = grc(s)
cs = c_char_p(s)
assert refcnt + 1 == grc(s)
assertSame(cs._objects, s)

print("ObjectsTestCase::test_c_char_p: ok")
