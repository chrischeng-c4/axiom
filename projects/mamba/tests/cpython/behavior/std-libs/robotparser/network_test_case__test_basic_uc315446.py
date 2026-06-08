# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "robotparser"
# dimension = "behavior"
# case = "network_test_case__test_basic_uc315446"
# subject = "cpython.test_robotparser.NetworkTestCase.test_basic"
# kind = "semantic"
# xfail = "auto-extracted CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_robotparser.py"
# status = "filled"
# ///
# mamba-xfail: auto-extracted CPython test; mamba promotion pending
import unittest, io
from test import test_robotparser
_suite = unittest.defaultTestLoader.loadTestsFromName("NetworkTestCase.test_basic", test_robotparser)
_result = unittest.TextTestRunner(stream=io.StringIO(), verbosity=0).run(_suite)
assert _result.wasSuccessful(), "CPython NetworkTestCase.test_basic did not pass"
print("NetworkTestCase::test_basic: ok")
