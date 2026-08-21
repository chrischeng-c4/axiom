# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "nntplib"
# dimension = "behavior"
# case = "local_server_tests__test_starttls"
# subject = "cpython.test_nntplib.LocalServerTests.test_starttls"
# kind = "semantic"
# xfail = "auto-extracted CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_nntplib.py"
# status = "filled"
# ///
# mamba-xfail: auto-extracted CPython test; mamba promotion pending
import unittest, io
from test import test_nntplib
_suite = unittest.defaultTestLoader.loadTestsFromName("LocalServerTests.test_starttls", test_nntplib)
_result = unittest.TextTestRunner(stream=io.StringIO(), verbosity=0).run(_suite)
assert _result.wasSuccessful(), "CPython LocalServerTests.test_starttls did not pass"
print("LocalServerTests::test_starttls: ok")
