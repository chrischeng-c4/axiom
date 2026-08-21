# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "nntplib"
# dimension = "behavior"
# case = "mock_socket_tests__test_service_permanently_unavailable"
# subject = "cpython.test_nntplib.MockSocketTests.test_service_permanently_unavailable"
# kind = "semantic"
# xfail = "auto-extracted CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_nntplib.py"
# status = "filled"
# ///
# mamba-xfail: auto-extracted CPython test; mamba promotion pending
import unittest, io
from test import test_nntplib
_suite = unittest.defaultTestLoader.loadTestsFromName("MockSocketTests.test_service_permanently_unavailable", test_nntplib)
_result = unittest.TextTestRunner(stream=io.StringIO(), verbosity=0).run(_suite)
assert _result.wasSuccessful(), "CPython MockSocketTests.test_service_permanently_unavailable did not pass"
print("MockSocketTests::test_service_permanently_unavailable: ok")
