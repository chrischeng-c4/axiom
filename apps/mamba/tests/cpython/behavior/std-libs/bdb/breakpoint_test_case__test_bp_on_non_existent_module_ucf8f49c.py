# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "bdb"
# dimension = "behavior"
# case = "breakpoint_test_case__test_bp_on_non_existent_module_ucf8f49c"
# subject = "cpython.test_bdb.BreakpointTestCase.test_bp_on_non_existent_module"
# kind = "semantic"
# xfail = "auto-extracted CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_bdb.py"
# status = "filled"
# ///
# mamba-xfail: auto-extracted CPython test; mamba promotion pending
import unittest, io
from test import test_bdb
_suite = unittest.defaultTestLoader.loadTestsFromName("BreakpointTestCase.test_bp_on_non_existent_module", test_bdb)
_result = unittest.TextTestRunner(stream=io.StringIO(), verbosity=0).run(_suite)
assert _result.wasSuccessful(), "CPython BreakpointTestCase.test_bp_on_non_existent_module did not pass"
print("BreakpointTestCase::test_bp_on_non_existent_module: ok")
