# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "parameters"
# dimension = "behavior"
# case = "simple_types_test_case__test_byref_pointerpointer_uc5681f5"
# subject = "cpython.test_parameters.SimpleTypesTestCase.test_byref_pointerpointer"
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
from ctypes import c_short, c_uint, c_int, c_long, pointer, POINTER, byref
LPLPINT = POINTER(POINTER(c_int))
LPLPINT.from_param(byref(pointer(c_int(42))))
try:
    LPLPINT.from_param(byref(pointer(c_short(22))))
    raise AssertionError('assertRaises: no raise')
except TypeError:
    pass
if c_int != c_long:
    try:
        LPLPINT.from_param(byref(pointer(c_long(22))))
        raise AssertionError('assertRaises: no raise')
    except TypeError:
        pass
try:
    LPLPINT.from_param(byref(pointer(c_uint(22))))
    raise AssertionError('assertRaises: no raise')
except TypeError:
    pass

print("SimpleTypesTestCase::test_byref_pointerpointer: ok")
