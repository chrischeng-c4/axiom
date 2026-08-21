# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "speedups"
# dimension = "behavior"
# case = "test_speedups__test_encode_basestring_ascii_uc73b798"
# subject = "cpython.test_speedups.TestSpeedups.test_encode_basestring_ascii"
# kind = "semantic"
# xfail = "auto-extracted CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_json/test_speedups.py"
# status = "filled"
# ///
# mamba-xfail: auto-extracted CPython test; mamba promotion pending
import unittest, io
from test.test_json import test_speedups
_suite = unittest.defaultTestLoader.loadTestsFromName("TestSpeedups.test_encode_basestring_ascii", test_speedups)
_result = unittest.TextTestRunner(stream=io.StringIO(), verbosity=0).run(_suite)
assert _result.wasSuccessful(), "CPython TestSpeedups.test_encode_basestring_ascii did not pass"
print("TestSpeedups::test_encode_basestring_ascii: ok")
