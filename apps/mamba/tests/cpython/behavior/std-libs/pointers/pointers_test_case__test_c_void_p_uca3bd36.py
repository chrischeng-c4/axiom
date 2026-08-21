# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "pointers"
# dimension = "behavior"
# case = "pointers_test_case__test_c_void_p_uca3bd36"
# subject = "cpython.test_pointers.PointersTestCase.test_c_void_p"
# kind = "semantic"
# xfail = "auto-extracted CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_ctypes/test_pointers.py"
# status = "filled"
# ///
# mamba-xfail: auto-extracted CPython test; mamba promotion pending
from ctypes import *
import _ctypes_test
if sizeof(c_void_p) == 4:
    assert c_void_p(4294967295).value == c_void_p(-1).value
    assert c_void_p(18446744073709551615).value == c_void_p(-1).value
elif sizeof(c_void_p) == 8:
    assert c_void_p(4294967295).value == 4294967295
    assert c_void_p(18446744073709551615).value == c_void_p(-1).value
    assert c_void_p(79228162514264337593543950335).value == c_void_p(-1).value
try:
    c_void_p(3.14)
    raise AssertionError('assertRaises: no raise')
except TypeError:
    pass
try:
    c_void_p(object())
    raise AssertionError('assertRaises: no raise')
except TypeError:
    pass

print("PointersTestCase::test_c_void_p: ok")
