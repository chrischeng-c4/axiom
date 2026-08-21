# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "ssl"
# dimension = "behavior"
# case = "context_tests__test_create_default_context"
# subject = "cpython.test_ssl.ContextTests.test_create_default_context"
# kind = "semantic"
# xfail = "auto-extracted CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_ssl.py"
# status = "filled"
# ///
# mamba-xfail: auto-extracted CPython test; mamba promotion pending
import unittest, io
from test import test_ssl
_suite = unittest.defaultTestLoader.loadTestsFromName("ContextTests.test_create_default_context", test_ssl)
_result = unittest.TextTestRunner(stream=io.StringIO(), verbosity=0).run(_suite)
assert _result.wasSuccessful(), "CPython ContextTests.test_create_default_context did not pass"
print("ContextTests::test_create_default_context: ok")
