# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "parameters"
# dimension = "behavior"
# case = "simple_types_test_case__test_subclasses_ucf157bf"
# subject = "cpython.test_parameters.SimpleTypesTestCase.test_subclasses"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_ctypes/test_parameters.py"
# status = "filled"
# ///
try:
    from _ctypes import set_conversion_mode
except ImportError:
    pass
else:
    self_prev_conv_mode = set_conversion_mode('ascii', 'strict')
from ctypes import c_void_p, c_char_p

class CVOIDP(c_void_p):

    def from_param(cls, value):
        return value * 2
    from_param = classmethod(from_param)

class CCHARP(c_char_p):

    def from_param(cls, value):
        return value * 4
    from_param = classmethod(from_param)
assert CVOIDP.from_param('abc') == 'abcabc'
assert CCHARP.from_param('abc') == 'abcabcabcabc'

print("SimpleTypesTestCase::test_subclasses: ok")
