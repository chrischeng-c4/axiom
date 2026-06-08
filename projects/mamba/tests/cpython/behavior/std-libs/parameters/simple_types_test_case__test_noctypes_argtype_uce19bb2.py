# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "parameters"
# dimension = "behavior"
# case = "simple_types_test_case__test_noctypes_argtype_uce19bb2"
# subject = "cpython.test_parameters.SimpleTypesTestCase.test_noctypes_argtype"
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
import _ctypes_test
from ctypes import CDLL, c_void_p, ArgumentError
func = CDLL(_ctypes_test.__file__)._testfunc_p_p
func.restype = c_void_p
try:
    setattr(func, 'argtypes', (object,))
    raise AssertionError('assertRaises: no raise')
except TypeError:
    pass

class Adapter:

    def from_param(cls, obj):
        return None
func.argtypes = (Adapter(),)
assert func(None) == None
assert func(object()) == None

class Adapter:

    def from_param(cls, obj):
        return obj
func.argtypes = (Adapter(),)
try:
    func(object())
    raise AssertionError('assertRaises: no raise')
except ArgumentError:
    pass
assert func(c_void_p(42)) == 42

class Adapter:

    def from_param(cls, obj):
        raise ValueError(obj)
func.argtypes = (Adapter(),)
try:
    func(99)
    raise AssertionError('assertRaises: no raise')
except ArgumentError:
    pass

print("SimpleTypesTestCase::test_noctypes_argtype: ok")
