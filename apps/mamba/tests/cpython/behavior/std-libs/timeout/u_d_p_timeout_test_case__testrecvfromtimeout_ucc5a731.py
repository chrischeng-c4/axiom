# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "timeout"
# dimension = "behavior"
# case = "u_d_p_timeout_test_case__testrecvfromtimeout_ucc5a731"
# subject = "cpython.test_timeout.UDPTimeoutTestCase.testRecvfromTimeout"
# kind = "semantic"
# xfail = "auto-extracted CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_timeout.py"
# status = "filled"
# ///
# mamba-xfail: auto-extracted CPython test; mamba promotion pending
import unittest, io
from test import test_timeout
_suite = unittest.defaultTestLoader.loadTestsFromName("UDPTimeoutTestCase.testRecvfromTimeout", test_timeout)
_result = unittest.TextTestRunner(stream=io.StringIO(), verbosity=0).run(_suite)
assert _result.wasSuccessful(), "CPython UDPTimeoutTestCase.testRecvfromTimeout did not pass"
print("UDPTimeoutTestCase::testRecvfromTimeout: ok")
