# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "telnetlib"
# dimension = "behavior"
# case = "general_tests__testbasic_uc9b3a0d"
# subject = "cpython.test_telnetlib.GeneralTests.testBasic"
# kind = "semantic"
# xfail = "auto-extracted CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_telnetlib.py"
# status = "filled"
# ///
# mamba-xfail: auto-extracted CPython test; mamba promotion pending
import unittest, io
from test import test_telnetlib
_suite = unittest.defaultTestLoader.loadTestsFromName("GeneralTests.testBasic", test_telnetlib)
_result = unittest.TextTestRunner(stream=io.StringIO(), verbosity=0).run(_suite)
assert _result.wasSuccessful(), "CPython GeneralTests.testBasic did not pass"
print("GeneralTests::testBasic: ok")
