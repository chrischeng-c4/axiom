# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "wsgiref"
# dimension = "behavior"
# case = "integration_tests__test_interrupted_write_uc25dfb8"
# subject = "cpython.test_wsgiref.IntegrationTests.test_interrupted_write"
# kind = "semantic"
# xfail = "auto-extracted CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_wsgiref.py"
# status = "filled"
# ///
# mamba-xfail: auto-extracted CPython test; mamba promotion pending
import unittest, io
from test import test_wsgiref
_suite = unittest.defaultTestLoader.loadTestsFromName("IntegrationTests.test_interrupted_write", test_wsgiref)
_result = unittest.TextTestRunner(stream=io.StringIO(), verbosity=0).run(_suite)
assert _result.wasSuccessful(), "CPython IntegrationTests.test_interrupted_write did not pass"
print("IntegrationTests::test_interrupted_write: ok")
