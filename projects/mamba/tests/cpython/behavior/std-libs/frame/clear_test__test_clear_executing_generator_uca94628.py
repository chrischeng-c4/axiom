# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "frame"
# dimension = "behavior"
# case = "clear_test__test_clear_executing_generator_uca94628"
# subject = "cpython.test_frame.ClearTest.test_clear_executing_generator"
# kind = "semantic"
# xfail = "auto-extracted CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_frame.py"
# status = "filled"
# ///
# mamba-xfail: auto-extracted CPython test; mamba promotion pending
import unittest, io
from test import test_frame
_suite = unittest.defaultTestLoader.loadTestsFromName("ClearTest.test_clear_executing_generator", test_frame)
_result = unittest.TextTestRunner(stream=io.StringIO(), verbosity=0).run(_suite)
assert _result.wasSuccessful(), "CPython ClearTest.test_clear_executing_generator did not pass"
print("ClearTest::test_clear_executing_generator: ok")
