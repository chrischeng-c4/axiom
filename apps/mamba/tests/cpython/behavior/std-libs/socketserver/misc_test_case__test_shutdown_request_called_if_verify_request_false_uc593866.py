# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "socketserver"
# dimension = "behavior"
# case = "misc_test_case__test_shutdown_request_called_if_verify_request_false_uc593866"
# subject = "cpython.test_socketserver.MiscTestCase.test_shutdown_request_called_if_verify_request_false"
# kind = "semantic"
# xfail = "auto-extracted CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_socketserver.py"
# status = "filled"
# ///
# mamba-xfail: auto-extracted CPython test; mamba promotion pending
import unittest, io
from test import test_socketserver
_suite = unittest.defaultTestLoader.loadTestsFromName("MiscTestCase.test_shutdown_request_called_if_verify_request_false", test_socketserver)
_result = unittest.TextTestRunner(stream=io.StringIO(), verbosity=0).run(_suite)
assert _result.wasSuccessful(), "CPython MiscTestCase.test_shutdown_request_called_if_verify_request_false did not pass"
print("MiscTestCase::test_shutdown_request_called_if_verify_request_false: ok")
