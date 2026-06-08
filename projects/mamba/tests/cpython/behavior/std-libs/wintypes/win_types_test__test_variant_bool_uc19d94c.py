# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "wintypes"
# dimension = "behavior"
# case = "win_types_test__test_variant_bool_uc19d94c"
# subject = "cpython.test_wintypes.WinTypesTest.test_variant_bool"
# kind = "semantic"
# xfail = "auto-extracted CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_ctypes/test_wintypes.py"
# status = "filled"
# ///
# mamba-xfail: auto-extracted CPython test; mamba promotion pending
from ctypes import *
from ctypes import wintypes

def assertIsSigned(ctype):
    assert ctype(-1).value < 0

def assertIsUnsigned(ctype):
    assert ctype(-1).value > 0
for true_value in (1, 32767, 32768, 65535, 65537):
    true = POINTER(c_int16)(c_int16(true_value))
    value = cast(true, POINTER(wintypes.VARIANT_BOOL))
    assert repr(value.contents) == 'VARIANT_BOOL(True)'
    vb = wintypes.VARIANT_BOOL()
    assert vb.value is False
    vb.value = True
    assert vb.value is True
    vb.value = true_value
    assert vb.value is True
for false_value in (0, 65536, 262144, 2 ** 33):
    false = POINTER(c_int16)(c_int16(false_value))
    value = cast(false, POINTER(wintypes.VARIANT_BOOL))
    assert repr(value.contents) == 'VARIANT_BOOL(False)'
for set_value in (65536, 262144, 2 ** 33):
    vb = wintypes.VARIANT_BOOL()
    vb.value = set_value
    assert vb.value is True
vb = wintypes.VARIANT_BOOL()
vb.value = [2, 3]
assert vb.value is True
vb.value = []
assert vb.value is False

print("WinTypesTest::test_variant_bool: ok")
