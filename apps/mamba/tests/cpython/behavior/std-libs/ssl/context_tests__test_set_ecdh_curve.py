# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "ssl"
# dimension = "behavior"
# case = "context_tests__test_set_ecdh_curve"
# subject = "cpython.test_ssl.ContextTests.test_set_ecdh_curve"
# kind = "semantic"
# xfail = "auto-extracted CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_ssl.py"
# status = "filled"
# ///
# mamba-xfail: auto-extracted CPython test; mamba promotion pending
import unittest, io
from test import test_ssl
_suite = unittest.defaultTestLoader.loadTestsFromName("ContextTests.test_set_ecdh_curve", test_ssl)
_result = unittest.TextTestRunner(stream=io.StringIO(), verbosity=0).run(_suite)
assert _result.wasSuccessful(), "CPython ContextTests.test_set_ecdh_curve did not pass"
print("ContextTests::test_set_ecdh_curve: ok")
