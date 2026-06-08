# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "speedups"
# dimension = "behavior"
# case = "test_encode__test_bad_str_encoder_uca95c55"
# subject = "cpython.test_speedups.TestEncode.test_bad_str_encoder"
# kind = "semantic"
# xfail = "auto-extracted CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_json/test_speedups.py"
# status = "filled"
# ///
# mamba-xfail: auto-extracted CPython test; mamba promotion pending
import unittest, io
from test.test_json import test_speedups
_suite = unittest.defaultTestLoader.loadTestsFromName("TestEncode.test_bad_str_encoder", test_speedups)
_result = unittest.TextTestRunner(stream=io.StringIO(), verbosity=0).run(_suite)
assert _result.wasSuccessful(), "CPython TestEncode.test_bad_str_encoder did not pass"
print("TestEncode::test_bad_str_encoder: ok")
