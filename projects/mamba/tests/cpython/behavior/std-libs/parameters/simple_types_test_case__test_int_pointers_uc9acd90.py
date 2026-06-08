# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "parameters"
# dimension = "behavior"
# case = "simple_types_test_case__test_int_pointers_uc9acd90"
# subject = "cpython.test_parameters.SimpleTypesTestCase.test_int_pointers"
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
from ctypes import c_short, c_uint, c_int, c_long, POINTER, pointer
LPINT = POINTER(c_int)
x = LPINT.from_param(pointer(c_int(42)))
assert x.contents.value == 42
assert LPINT(c_int(42)).contents.value == 42
assert LPINT.from_param(None) == None
if c_int != c_long:
    try:
        LPINT.from_param(pointer(c_long(42)))
        raise AssertionError('assertRaises: no raise')
    except TypeError:
        pass
try:
    LPINT.from_param(pointer(c_uint(42)))
    raise AssertionError('assertRaises: no raise')
except TypeError:
    pass
try:
    LPINT.from_param(pointer(c_short(42)))
    raise AssertionError('assertRaises: no raise')
except TypeError:
    pass

print("SimpleTypesTestCase::test_int_pointers: ok")
