# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "memfunctions"
# dimension = "behavior"
# case = "mem_functions_test__test_string_at_uc7d59a4"
# subject = "cpython.test_memfunctions.MemFunctionsTest.test_string_at"
# kind = "semantic"
# xfail = "auto-extracted CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_ctypes/test_memfunctions.py"
# status = "filled"
# ///
# mamba-xfail: auto-extracted CPython test; mamba promotion pending
import unittest, io
from test.test_ctypes import test_memfunctions
_suite = unittest.defaultTestLoader.loadTestsFromName("MemFunctionsTest.test_string_at", test_memfunctions)
_result = unittest.TextTestRunner(stream=io.StringIO(), verbosity=0).run(_suite)
assert _result.wasSuccessful(), "CPython MemFunctionsTest.test_string_at did not pass"
print("MemFunctionsTest::test_string_at: ok")
