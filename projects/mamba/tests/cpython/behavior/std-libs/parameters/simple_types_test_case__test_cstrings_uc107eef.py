# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "parameters"
# dimension = "behavior"
# case = "simple_types_test_case__test_cstrings_uc107eef"
# subject = "cpython.test_parameters.SimpleTypesTestCase.test_cstrings"
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
from ctypes import c_char_p
s = b'123'
assert c_char_p.from_param(s)._obj is s
assert c_char_p.from_param(b'123')._obj == b'123'
try:
    c_char_p.from_param('123ÿ')
    raise AssertionError('assertRaises: no raise')
except TypeError:
    pass
try:
    c_char_p.from_param(42)
    raise AssertionError('assertRaises: no raise')
except TypeError:
    pass
a = c_char_p(b'123')
assert c_char_p.from_param(a) is a

print("SimpleTypesTestCase::test_cstrings: ok")
