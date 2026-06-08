# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "result"
# dimension = "behavior"
# case = "test_output_buffering__testBufferSetUpModule_DoModuleCleanups"
# subject = "cpython.test_result.TestOutputBuffering.testBufferSetUpModule_DoModuleCleanups"
# kind = "semantic"
# xfail = "auto-extracted CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_unittest/test_result.py"
# status = "filled"
# ///
# mamba-xfail: auto-extracted CPython test; mamba promotion pending
import unittest, io
from test.test_unittest import test_result
_suite = unittest.defaultTestLoader.loadTestsFromName("TestOutputBuffering.testBufferSetUpModule_DoModuleCleanups", test_result)
_result = unittest.TextTestRunner(stream=io.StringIO(), verbosity=0).run(_suite)
assert _result.wasSuccessful(), "CPython TestOutputBuffering.testBufferSetUpModule_DoModuleCleanups did not pass"
print("TestOutputBuffering::testBufferSetUpModule_DoModuleCleanups: ok")
