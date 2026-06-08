# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "urllib2net"
# dimension = "behavior"
# case = "timeout_test__test_ftp_timeout_uc7f1498"
# subject = "cpython.test_urllib2net.TimeoutTest.test_ftp_timeout"
# kind = "semantic"
# xfail = "auto-extracted CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_urllib2net.py"
# status = "filled"
# ///
# mamba-xfail: auto-extracted CPython test; mamba promotion pending
import unittest, io
from test import test_urllib2net
_suite = unittest.defaultTestLoader.loadTestsFromName("TimeoutTest.test_ftp_timeout", test_urllib2net)
_result = unittest.TextTestRunner(stream=io.StringIO(), verbosity=0).run(_suite)
assert _result.wasSuccessful(), "CPython TimeoutTest.test_ftp_timeout did not pass"
print("TimeoutTest::test_ftp_timeout: ok")
