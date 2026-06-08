# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "bdb"
# dimension = "behavior"
# case = "breakpoint_test_case__test_disabled_temporary_bp_uc82f860"
# subject = "cpython.test_bdb.BreakpointTestCase.test_disabled_temporary_bp"
# kind = "semantic"
# xfail = "auto-extracted CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_bdb.py"
# status = "filled"
# ///
# mamba-xfail: auto-extracted CPython test; mamba promotion pending
import unittest, io
from test import test_bdb
_suite = unittest.defaultTestLoader.loadTestsFromName("BreakpointTestCase.test_disabled_temporary_bp", test_bdb)
_result = unittest.TextTestRunner(stream=io.StringIO(), verbosity=0).run(_suite)
assert _result.wasSuccessful(), "CPython BreakpointTestCase.test_disabled_temporary_bp did not pass"
print("BreakpointTestCase::test_disabled_temporary_bp: ok")
