# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "byteswap"
# dimension = "behavior"
# case = "test__test_struct_struct_uca1ca66"
# subject = "cpython.test_byteswap.Test.test_struct_struct"
# kind = "semantic"
# xfail = "auto-extracted CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_ctypes/test_byteswap.py"
# status = "filled"
# ///
# mamba-xfail: auto-extracted CPython test; mamba promotion pending
import unittest, io
from test.test_ctypes import test_byteswap
_suite = unittest.defaultTestLoader.loadTestsFromName("Test.test_struct_struct", test_byteswap)
_result = unittest.TextTestRunner(stream=io.StringIO(), verbosity=0).run(_suite)
assert _result.wasSuccessful(), "CPython Test.test_struct_struct did not pass"
print("Test::test_struct_struct: ok")
