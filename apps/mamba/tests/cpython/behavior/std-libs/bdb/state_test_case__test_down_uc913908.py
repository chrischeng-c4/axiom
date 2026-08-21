# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "bdb"
# dimension = "behavior"
# case = "state_test_case__test_down_uc913908"
# subject = "cpython.test_bdb.StateTestCase.test_down"
# kind = "semantic"
# xfail = "auto-extracted CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_bdb.py"
# status = "filled"
# ///
# mamba-xfail: auto-extracted CPython test; mamba promotion pending
import unittest, io
from test import test_bdb
_suite = unittest.defaultTestLoader.loadTestsFromName("StateTestCase.test_down", test_bdb)
_result = unittest.TextTestRunner(stream=io.StringIO(), verbosity=0).run(_suite)
assert _result.wasSuccessful(), "CPython StateTestCase.test_down did not pass"
print("StateTestCase::test_down: ok")
