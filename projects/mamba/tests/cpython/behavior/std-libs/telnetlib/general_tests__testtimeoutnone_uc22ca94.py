# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "telnetlib"
# dimension = "behavior"
# case = "general_tests__testtimeoutnone_uc22ca94"
# subject = "cpython.test_telnetlib.GeneralTests.testTimeoutNone"
# kind = "semantic"
# xfail = "auto-extracted CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_telnetlib.py"
# status = "filled"
# ///
# mamba-xfail: auto-extracted CPython test; mamba promotion pending
import unittest, io
from test import test_telnetlib
_suite = unittest.defaultTestLoader.loadTestsFromName("GeneralTests.testTimeoutNone", test_telnetlib)
_result = unittest.TextTestRunner(stream=io.StringIO(), verbosity=0).run(_suite)
assert _result.wasSuccessful(), "CPython GeneralTests.testTimeoutNone did not pass"
print("GeneralTests::testTimeoutNone: ok")
