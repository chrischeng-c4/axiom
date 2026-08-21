# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "speedups"
# dimension = "behavior"
# case = "test_encode__test_bad_markers_argument_to_encoder_uc7b00d7"
# subject = "cpython.test_speedups.TestEncode.test_bad_markers_argument_to_encoder"
# kind = "semantic"
# xfail = "auto-extracted CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_json/test_speedups.py"
# status = "filled"
# ///
# mamba-xfail: auto-extracted CPython test; mamba promotion pending
import unittest, io
from test.test_json import test_speedups
_suite = unittest.defaultTestLoader.loadTestsFromName("TestEncode.test_bad_markers_argument_to_encoder", test_speedups)
_result = unittest.TextTestRunner(stream=io.StringIO(), verbosity=0).run(_suite)
assert _result.wasSuccessful(), "CPython TestEncode.test_bad_markers_argument_to_encoder did not pass"
print("TestEncode::test_bad_markers_argument_to_encoder: ok")
