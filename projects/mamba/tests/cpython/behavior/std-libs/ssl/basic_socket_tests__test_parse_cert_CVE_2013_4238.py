# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "ssl"
# dimension = "behavior"
# case = "basic_socket_tests__test_parse_cert_CVE_2013_4238"
# subject = "cpython.test_ssl.BasicSocketTests.test_parse_cert_CVE_2013_4238"
# kind = "semantic"
# xfail = "auto-extracted CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_ssl.py"
# status = "filled"
# ///
# mamba-xfail: auto-extracted CPython test; mamba promotion pending
import unittest, io
from test import test_ssl
_suite = unittest.defaultTestLoader.loadTestsFromName("BasicSocketTests.test_parse_cert_CVE_2013_4238", test_ssl)
_result = unittest.TextTestRunner(stream=io.StringIO(), verbosity=0).run(_suite)
assert _result.wasSuccessful(), "CPython BasicSocketTests.test_parse_cert_CVE_2013_4238 did not pass"
print("BasicSocketTests::test_parse_cert_CVE_2013_4238: ok")
