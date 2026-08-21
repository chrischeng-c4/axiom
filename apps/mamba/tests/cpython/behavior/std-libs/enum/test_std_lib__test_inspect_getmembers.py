# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "enum"
# dimension = "behavior"
# case = "test_std_lib__test_inspect_getmembers"
# subject = "cpython.test_enum.TestStdLib.test_inspect_getmembers"
# kind = "semantic"
# xfail = "auto-extracted CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_enum.py"
# status = "filled"
# ///
# mamba-xfail: auto-extracted CPython test; mamba promotion pending
import unittest, io
from test import test_enum
_suite = unittest.defaultTestLoader.loadTestsFromName("TestStdLib.test_inspect_getmembers", test_enum)
_result = unittest.TextTestRunner(stream=io.StringIO(), verbosity=0).run(_suite)
assert _result.wasSuccessful(), "CPython TestStdLib.test_inspect_getmembers did not pass"
print("TestStdLib::test_inspect_getmembers: ok")
