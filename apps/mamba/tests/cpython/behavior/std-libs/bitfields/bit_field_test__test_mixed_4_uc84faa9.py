# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "bitfields"
# dimension = "behavior"
# case = "bit_field_test__test_mixed_4_uc84faa9"
# subject = "cpython.test_bitfields.BitFieldTest.test_mixed_4"
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

class X(Structure):
    _fields_ = [('a', c_short, 4), ('b', c_short, 4), ('c', c_int, 24), ('d', c_short, 4), ('e', c_short, 4), ('f', c_int, 24)]
if os.name == 'nt':
    assert sizeof(X) == sizeof(c_int) * 4
else:
    assert sizeof(X) == sizeof(c_int) * 2

print("BitFieldTest::test_mixed_4: ok")
