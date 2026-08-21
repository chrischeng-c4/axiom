# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "urllibnet"
# dimension = "behavior"
# case = "u_r_l_timeout_test__testurlread_uc5f0fd9"
# subject = "cpython.test_urllibnet.URLTimeoutTest.testURLread"
# kind = "semantic"
# xfail = "auto-extracted CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_urllibnet.py"
# status = "filled"
# ///
# mamba-xfail: auto-extracted CPython test; mamba promotion pending
import unittest, io
from test import test_urllibnet
_suite = unittest.defaultTestLoader.loadTestsFromName("URLTimeoutTest.testURLread", test_urllibnet)
_result = unittest.TextTestRunner(stream=io.StringIO(), verbosity=0).run(_suite)
assert _result.wasSuccessful(), "CPython URLTimeoutTest.testURLread did not pass"
print("URLTimeoutTest::testURLread: ok")
