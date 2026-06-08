# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "pipes"
# dimension = "behavior"
# case = "simple_pipe_tests__testsimplepipe1_uce11970"
# subject = "cpython.test_pipes.SimplePipeTests.testSimplePipe1"
# kind = "semantic"
# xfail = "auto-extracted CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_pipes.py"
# status = "filled"
# ///
# mamba-xfail: auto-extracted CPython test; mamba promotion pending
import unittest, io
from test import test_pipes
_suite = unittest.defaultTestLoader.loadTestsFromName("SimplePipeTests.testSimplePipe1", test_pipes)
_result = unittest.TextTestRunner(stream=io.StringIO(), verbosity=0).run(_suite)
assert _result.wasSuccessful(), "CPython SimplePipeTests.testSimplePipe1 did not pass"
print("SimplePipeTests::testSimplePipe1: ok")
