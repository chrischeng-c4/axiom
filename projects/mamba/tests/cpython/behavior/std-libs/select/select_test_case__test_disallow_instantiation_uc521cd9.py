# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "select"
# dimension = "behavior"
# case = "select_test_case__test_disallow_instantiation_uc521cd9"
# subject = "cpython.test_select.SelectTestCase.test_disallow_instantiation"
# kind = "semantic"
# xfail = "auto-extracted CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_select.py"
# status = "filled"
# ///
# mamba-xfail: auto-extracted CPython test; mamba promotion pending
import unittest, io
from test import test_select
_suite = unittest.defaultTestLoader.loadTestsFromName("SelectTestCase.test_disallow_instantiation", test_select)
_result = unittest.TextTestRunner(stream=io.StringIO(), verbosity=0).run(_suite)
assert _result.wasSuccessful(), "CPython SelectTestCase.test_disallow_instantiation did not pass"
print("SelectTestCase::test_disallow_instantiation: ok")
