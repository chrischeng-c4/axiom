# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "timeout"
# dimension = "behavior"
# case = "t_c_p_timeout_test_case__testconnecttimeout_ucb0a2b1"
# subject = "cpython.test_timeout.TCPTimeoutTestCase.testConnectTimeout"
# kind = "semantic"
# xfail = "auto-extracted CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_timeout.py"
# status = "filled"
# ///
# mamba-xfail: auto-extracted CPython test; mamba promotion pending
import unittest, io
from test import test_timeout
_suite = unittest.defaultTestLoader.loadTestsFromName("TCPTimeoutTestCase.testConnectTimeout", test_timeout)
_result = unittest.TextTestRunner(stream=io.StringIO(), verbosity=0).run(_suite)
assert _result.wasSuccessful(), "CPython TCPTimeoutTestCase.testConnectTimeout did not pass"
print("TCPTimeoutTestCase::testConnectTimeout: ok")
