# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "csv"
# dimension = "behavior"
# case = "misc_test_case__test_disallow_instantiation_ucfcf3b6"
# subject = "cpython.test_csv.MiscTestCase.test_disallow_instantiation"
# kind = "semantic"
# xfail = "auto-extracted CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_csv.py"
# status = "filled"
# ///
# mamba-xfail: auto-extracted CPython test; mamba promotion pending
import unittest, io
from test import test_csv
_suite = unittest.defaultTestLoader.loadTestsFromName("MiscTestCase.test_disallow_instantiation", test_csv)
_result = unittest.TextTestRunner(stream=io.StringIO(), verbosity=0).run(_suite)
assert _result.wasSuccessful(), "CPython MiscTestCase.test_disallow_instantiation did not pass"
print("MiscTestCase::test_disallow_instantiation: ok")
