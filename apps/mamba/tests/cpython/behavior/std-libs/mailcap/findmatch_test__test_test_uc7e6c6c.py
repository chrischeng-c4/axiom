# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "mailcap"
# dimension = "behavior"
# case = "findmatch_test__test_test_uc7e6c6c"
# subject = "cpython.test_mailcap.FindmatchTest.test_test"
# kind = "semantic"
# xfail = "auto-extracted CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_mailcap.py"
# status = "filled"
# ///
# mamba-xfail: auto-extracted CPython test; mamba promotion pending
import unittest, io
from test import test_mailcap
_suite = unittest.defaultTestLoader.loadTestsFromName("FindmatchTest.test_test", test_mailcap)
_result = unittest.TextTestRunner(stream=io.StringIO(), verbosity=0).run(_suite)
assert _result.wasSuccessful(), "CPython FindmatchTest.test_test did not pass"
print("FindmatchTest::test_test: ok")
