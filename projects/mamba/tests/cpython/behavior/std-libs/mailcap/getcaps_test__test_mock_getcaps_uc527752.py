# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "mailcap"
# dimension = "behavior"
# case = "getcaps_test__test_mock_getcaps_uc527752"
# subject = "cpython.test_mailcap.GetcapsTest.test_mock_getcaps"
# kind = "semantic"
# xfail = "auto-extracted CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_mailcap.py"
# status = "filled"
# ///
# mamba-xfail: auto-extracted CPython test; mamba promotion pending
import unittest, io
from test import test_mailcap
_suite = unittest.defaultTestLoader.loadTestsFromName("GetcapsTest.test_mock_getcaps", test_mailcap)
_result = unittest.TextTestRunner(stream=io.StringIO(), verbosity=0).run(_suite)
assert _result.wasSuccessful(), "CPython GetcapsTest.test_mock_getcaps did not pass"
print("GetcapsTest::test_mock_getcaps: ok")
