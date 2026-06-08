# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "parameters"
# dimension = "behavior"
# case = "simple_types_test_case__test_abstract_uc83bc19"
# subject = "cpython.test_parameters.SimpleTypesTestCase.test_abstract"
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
from ctypes import Array, Structure, Union, _Pointer, _SimpleCData, _CFuncPtr
try:
    Array.from_param(42)
    raise AssertionError('assertRaises: no raise')
except TypeError:
    pass
try:
    Structure.from_param(42)
    raise AssertionError('assertRaises: no raise')
except TypeError:
    pass
try:
    Union.from_param(42)
    raise AssertionError('assertRaises: no raise')
except TypeError:
    pass
try:
    _CFuncPtr.from_param(42)
    raise AssertionError('assertRaises: no raise')
except TypeError:
    pass
try:
    _Pointer.from_param(42)
    raise AssertionError('assertRaises: no raise')
except TypeError:
    pass
try:
    _SimpleCData.from_param(42)
    raise AssertionError('assertRaises: no raise')
except TypeError:
    pass

print("SimpleTypesTestCase::test_abstract: ok")
