# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "speedups"
# dimension = "behavior"
# case = "test_decode__test_bad_bool_args_uc8dde6b"
# subject = "cpython.test_speedups.TestDecode.test_bad_bool_args"
# kind = "semantic"
# xfail = "auto-extracted CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_json/test_speedups.py"
# status = "filled"
# ///
# mamba-xfail: auto-extracted CPython test; mamba promotion pending
import unittest, io
from test.test_json import test_speedups
_suite = unittest.defaultTestLoader.loadTestsFromName("TestDecode.test_bad_bool_args", test_speedups)
_result = unittest.TextTestRunner(stream=io.StringIO(), verbosity=0).run(_suite)
assert _result.wasSuccessful(), "CPython TestDecode.test_bad_bool_args did not pass"
print("TestDecode::test_bad_bool_args: ok")
