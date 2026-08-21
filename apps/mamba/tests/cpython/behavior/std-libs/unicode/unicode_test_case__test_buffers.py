# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "unicode"
# dimension = "behavior"
# case = "unicode_test_case__test_buffers"
# subject = "cpython.test_unicode.UnicodeTestCase.test_buffers"
# kind = "semantic"
# xfail = "auto-extracted CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_ctypes/test_unicode.py"
# status = "filled"
# ///
# mamba-xfail: auto-extracted CPython test; mamba promotion pending
import ctypes
import _ctypes_test
buf = ctypes.create_unicode_buffer('abc')
assert len(buf) == 3 + 1
buf = ctypes.create_unicode_buffer('abäöü')
assert buf[:] == 'abäöü\x00'
assert buf[:] == 'abäöü\x00'
assert buf[::-1] == '\x00üöäba'
assert buf[::2] == 'aäü'
assert buf[6:5:-1] == ''

print("UnicodeTestCase::test_buffers: ok")
