# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "timeout"
# dimension = "behavior"
# case = "t_c_p_timeout_test_case__testaccepttimeout_uc9dba35"
# subject = "cpython.test_timeout.TCPTimeoutTestCase.testAcceptTimeout"
# kind = "semantic"
# xfail = "auto-extracted CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_timeout.py"
# status = "filled"
# ///
# mamba-xfail: auto-extracted CPython test; mamba promotion pending
import unittest, io
from test import test_timeout
_suite = unittest.defaultTestLoader.loadTestsFromName("TCPTimeoutTestCase.testAcceptTimeout", test_timeout)
_result = unittest.TextTestRunner(stream=io.StringIO(), verbosity=0).run(_suite)
assert _result.wasSuccessful(), "CPython TCPTimeoutTestCase.testAcceptTimeout did not pass"
print("TCPTimeoutTestCase::testAcceptTimeout: ok")
