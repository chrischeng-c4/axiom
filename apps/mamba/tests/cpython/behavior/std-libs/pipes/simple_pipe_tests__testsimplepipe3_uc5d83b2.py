# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "pipes"
# dimension = "behavior"
# case = "simple_pipe_tests__testsimplepipe3_uc5d83b2"
# subject = "cpython.test_pipes.SimplePipeTests.testSimplePipe3"
# kind = "semantic"
# xfail = "auto-extracted CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_pipes.py"
# status = "filled"
# ///
# mamba-xfail: auto-extracted CPython test; mamba promotion pending
import unittest, io
from test import test_pipes
_suite = unittest.defaultTestLoader.loadTestsFromName("SimplePipeTests.testSimplePipe3", test_pipes)
_result = unittest.TextTestRunner(stream=io.StringIO(), verbosity=0).run(_suite)
assert _result.wasSuccessful(), "CPython SimplePipeTests.testSimplePipe3 did not pass"
print("SimplePipeTests::testSimplePipe3: ok")
