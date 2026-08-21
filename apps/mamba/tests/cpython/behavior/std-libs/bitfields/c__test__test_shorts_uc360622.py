# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "bitfields"
# dimension = "behavior"
# case = "c__test__test_shorts_uc360622"
# subject = "cpython.test_bitfields.C_Test.test_shorts"
# kind = "semantic"
# xfail = "auto-extracted CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_ctypes/test_bitfields.py"
# status = "filled"
# ///
# mamba-xfail: auto-extracted CPython test; mamba promotion pending
import unittest, io
from test.test_ctypes import test_bitfields
_suite = unittest.defaultTestLoader.loadTestsFromName("C_Test.test_shorts", test_bitfields)
_result = unittest.TextTestRunner(stream=io.StringIO(), verbosity=0).run(_suite)
assert _result.wasSuccessful(), "CPython C_Test.test_shorts did not pass"
print("C_Test::test_shorts: ok")
