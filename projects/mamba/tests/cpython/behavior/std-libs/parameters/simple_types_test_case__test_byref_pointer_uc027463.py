# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "parameters"
# dimension = "behavior"
# case = "simple_types_test_case__test_byref_pointer_uc027463"
# subject = "cpython.test_parameters.SimpleTypesTestCase.test_byref_pointer"
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
from ctypes import c_short, c_uint, c_int, c_long, POINTER, byref
LPINT = POINTER(c_int)
LPINT.from_param(byref(c_int(42)))
try:
    LPINT.from_param(byref(c_short(22)))
    raise AssertionError('assertRaises: no raise')
except TypeError:
    pass
if c_int != c_long:
    try:
        LPINT.from_param(byref(c_long(22)))
        raise AssertionError('assertRaises: no raise')
    except TypeError:
        pass
try:
    LPINT.from_param(byref(c_uint(22)))
    raise AssertionError('assertRaises: no raise')
except TypeError:
    pass

print("SimpleTypesTestCase::test_byref_pointer: ok")
