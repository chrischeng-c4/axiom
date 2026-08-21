# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "dis"
# dimension = "behavior"
# case = "dis_tests__test_bug_1333982_uc4d2495"
# subject = "cpython.test_dis.DisTests.test_bug_1333982"
# kind = "semantic"
# xfail = "auto-extracted CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_dis.py"
# status = "filled"
# ///
# mamba-xfail: auto-extracted CPython test; mamba promotion pending
import unittest, io
from test import test_dis
_suite = unittest.defaultTestLoader.loadTestsFromName("DisTests.test_bug_1333982", test_dis)
_result = unittest.TextTestRunner(stream=io.StringIO(), verbosity=0).run(_suite)
assert _result.wasSuccessful(), "CPython DisTests.test_bug_1333982 did not pass"
print("DisTests::test_bug_1333982: ok")
