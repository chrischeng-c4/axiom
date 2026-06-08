# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "unicode"
# dimension = "behavior"
# case = "unicode_test_case__test_embedded_null"
# subject = "cpython.test_unicode.UnicodeTestCase.test_embedded_null"
# kind = "semantic"
# xfail = "auto-extracted CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_ctypes/test_unicode.py"
# status = "filled"
# ///
# mamba-xfail: auto-extracted CPython test; mamba promotion pending
import ctypes
import _ctypes_test

class TestStruct(ctypes.Structure):
    _fields_ = [('unicode', ctypes.c_wchar_p)]
t = TestStruct()
t.unicode = 'foo\x00bar\x00\x00'

print("UnicodeTestCase::test_embedded_null: ok")
