# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "cast"
# dimension = "behavior"
# case = "test__test_wchar_p_uc48754b"
# subject = "cpython.test_cast.Test.test_wchar_p"
# kind = "semantic"
# xfail = "auto-extracted CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_ctypes/test_cast.py"
# status = "filled"
# ///
# mamba-xfail: auto-extracted CPython test; mamba promotion pending
import unittest, io
from test.test_ctypes import test_cast
_suite = unittest.defaultTestLoader.loadTestsFromName("Test.test_wchar_p", test_cast)
_result = unittest.TextTestRunner(stream=io.StringIO(), verbosity=0).run(_suite)
assert _result.wasSuccessful(), "CPython Test.test_wchar_p did not pass"
print("Test::test_wchar_p: ok")
