# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "robotparser"
# dimension = "behavior"
# case = "network_test_case__test_can_fetch_ucd0c4dd"
# subject = "cpython.test_robotparser.NetworkTestCase.test_can_fetch"
# kind = "semantic"
# xfail = "auto-extracted CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_robotparser.py"
# status = "filled"
# ///
# mamba-xfail: auto-extracted CPython test; mamba promotion pending
import unittest, io
from test import test_robotparser
_suite = unittest.defaultTestLoader.loadTestsFromName("NetworkTestCase.test_can_fetch", test_robotparser)
_result = unittest.TextTestRunner(stream=io.StringIO(), verbosity=0).run(_suite)
assert _result.wasSuccessful(), "CPython NetworkTestCase.test_can_fetch did not pass"
print("NetworkTestCase::test_can_fetch: ok")
