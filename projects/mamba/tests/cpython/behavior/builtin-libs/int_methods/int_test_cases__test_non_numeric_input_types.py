# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "builtin-libs"
# lib = "int_methods"
# dimension = "behavior"
# case = "int_test_cases__test_non_numeric_input_types"
# subject = "cpython.test_int.IntTestCases.test_non_numeric_input_types"
# kind = "semantic"
# xfail = "auto-ported CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_int.py"
# status = "filled"
# ///
# mamba-xfail: auto-ported CPython test; mamba promotion pending
# Auto-ported from CPython 3.12 test_int.py::IntTestCases::test_non_numeric_input_types
"""Auto-ported test: IntTestCases::test_non_numeric_input_types (CPython 3.12 oracle)."""


import sys
import time
import unittest
from unittest import mock
from test import support
from test.test_grammar import VALID_UNDERSCORE_LITERALS, INVALID_UNDERSCORE_LITERALS


try:
    import _pylong
except ImportError:
    _pylong = None

L = [('0', 0), ('1', 1), ('9', 9), ('10', 10), ('99', 99), ('100', 100), ('314', 314), (' 314', 314), ('314 ', 314), ('  \t\t  314  \t\t  ', 314), (repr(sys.maxsize), sys.maxsize), ('  1x', ValueError), ('  1  ', 1), ('  1\x02  ', ValueError), ('', ValueError), (' ', ValueError), ('  \t\t  ', ValueError), ('Ȁ', ValueError)]

class IntSubclass(int):
    pass


# --- test body ---
class CustomStr(str):
    pass

class CustomBytes(bytes):
    pass

class CustomByteArray(bytearray):
    pass
factories = [bytes, bytearray, lambda b: CustomStr(b.decode()), CustomBytes, CustomByteArray, memoryview]
try:
    from array import array
except ImportError:
    pass
else:
    factories.append(lambda b: array('B', b))
for f in factories:
    x = f(b'100')

    assert int(x) == 100
    if isinstance(x, (str, bytes, bytearray)):

        assert int(x, 2) == 4
    else:
        msg = "can't convert non-string"
        try:
            int(x, 2)
            raise AssertionError('expected TypeError')
        except TypeError as _aR_e:
            import re as _re_aR
            assert _re_aR.search(msg, str(_aR_e))
    try:
        int(f(b'A' * 16))
        raise AssertionError('expected ValueError')
    except ValueError as _aR_e:
        import re as _re_aR
        assert _re_aR.search('invalid literal', str(_aR_e))
print("IntTestCases::test_non_numeric_input_types: ok")
