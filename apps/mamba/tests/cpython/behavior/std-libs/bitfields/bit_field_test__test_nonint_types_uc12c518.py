# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "bitfields"
# dimension = "behavior"
# case = "bit_field_test__test_nonint_types_uc12c518"
# subject = "cpython.test_bitfields.BitFieldTest.test_nonint_types"
# kind = "semantic"
# xfail = "auto-extracted CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_ctypes/test_bitfields.py"
# status = "filled"
# ///
# mamba-xfail: auto-extracted CPython test; mamba promotion pending
from ctypes import *
import os
import _ctypes_test

def fail_fields(*fields):
    return get_except(type(Structure), 'X', (), {'_fields_': fields})

def get_except(func, *args, **kw):
    try:
        func(*args, **kw)
    except Exception as detail:
        return (detail.__class__, str(detail))
result = fail_fields(('a', c_char_p, 1))
assert result == (TypeError, 'bit fields not allowed for type c_char_p')
result = fail_fields(('a', c_void_p, 1))
assert result == (TypeError, 'bit fields not allowed for type c_void_p')
if c_int != c_long:
    result = fail_fields(('a', POINTER(c_int), 1))
    assert result == (TypeError, 'bit fields not allowed for type LP_c_int')
result = fail_fields(('a', c_char, 1))
assert result == (TypeError, 'bit fields not allowed for type c_char')

class Dummy(Structure):
    _fields_ = []
result = fail_fields(('a', Dummy, 1))
assert result == (TypeError, 'bit fields not allowed for type Dummy')

print("BitFieldTest::test_nonint_types: ok")
