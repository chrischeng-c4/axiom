# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "unicode"
# dimension = "behavior"
# case = "unicode_test_case__test_wcslen"
# subject = "cpython.test_unicode.UnicodeTestCase.test_wcslen"
# kind = "semantic"
# xfail = "auto-extracted CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_ctypes/test_unicode.py"
# status = "filled"
# ///
# mamba-xfail: auto-extracted CPython test; mamba promotion pending
import ctypes
import _ctypes_test
dll = ctypes.CDLL(_ctypes_test.__file__)
wcslen = dll.my_wcslen
wcslen.argtypes = [ctypes.c_wchar_p]
assert wcslen('abc') == 3
assert wcslen('ab⁰') == 3
try:
    wcslen(b'ab\xe4')
    raise AssertionError('assertRaises: no raise')
except ctypes.ArgumentError:
    pass

print("UnicodeTestCase::test_wcslen: ok")
