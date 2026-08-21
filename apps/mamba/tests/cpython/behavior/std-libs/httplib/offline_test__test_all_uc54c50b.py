# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "httplib"
# dimension = "behavior"
# case = "offline_test__test_all_uc54c50b"
# subject = "cpython.test_httplib.OfflineTest.test_all"
# kind = "semantic"
# xfail = "auto-extracted CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_httplib.py"
# status = "filled"
# ///
# mamba-xfail: auto-extracted CPython test; mamba promotion pending
import unittest, io
from test import test_httplib
_suite = unittest.defaultTestLoader.loadTestsFromName("OfflineTest.test_all", test_httplib)
_result = unittest.TextTestRunner(stream=io.StringIO(), verbosity=0).run(_suite)
assert _result.wasSuccessful(), "CPython OfflineTest.test_all did not pass"
print("OfflineTest::test_all: ok")
