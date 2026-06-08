# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "parameters"
# dimension = "behavior"
# case = "simple_types_test_case__test_array_pointers_uc1d7be6"
# subject = "cpython.test_parameters.SimpleTypesTestCase.test_array_pointers"
# kind = "semantic"
# xfail = "auto-extracted CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_ctypes/test_parameters.py"
# status = "filled"
# ///
# mamba-xfail: auto-extracted CPython test; mamba promotion pending
try:
    from _ctypes import set_conversion_mode
except ImportError:
    pass
else:
    self_prev_conv_mode = set_conversion_mode('ascii', 'strict')
from ctypes import c_short, c_uint, c_int, c_long, POINTER
INTARRAY = c_int * 3
ia = INTARRAY()
assert len(ia) == 3
assert [ia[i] for i in range(3)] == [0, 0, 0]
LPINT = POINTER(c_int)
LPINT.from_param((c_int * 3)())
try:
    LPINT.from_param(c_short * 3)
    raise AssertionError('assertRaises: no raise')
except TypeError:
    pass
try:
    LPINT.from_param(c_long * 3)
    raise AssertionError('assertRaises: no raise')
except TypeError:
    pass
try:
    LPINT.from_param(c_uint * 3)
    raise AssertionError('assertRaises: no raise')
except TypeError:
    pass

print("SimpleTypesTestCase::test_array_pointers: ok")
