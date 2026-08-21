# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "ssl"
# dimension = "behavior"
# case = "simple_background_tests__test_get_server_certificate_fail"
# subject = "cpython.test_ssl.SimpleBackgroundTests.test_get_server_certificate_fail"
# kind = "semantic"
# xfail = "auto-extracted CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_ssl.py"
# status = "filled"
# ///
# mamba-xfail: auto-extracted CPython test; mamba promotion pending
import unittest, io
from test import test_ssl
_suite = unittest.defaultTestLoader.loadTestsFromName("SimpleBackgroundTests.test_get_server_certificate_fail", test_ssl)
_result = unittest.TextTestRunner(stream=io.StringIO(), verbosity=0).run(_suite)
assert _result.wasSuccessful(), "CPython SimpleBackgroundTests.test_get_server_certificate_fail did not pass"
print("SimpleBackgroundTests::test_get_server_certificate_fail: ok")
