# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "pyexpat"
# dimension = "behavior"
# case = "reparse_deferral_test__test_reparse_deferral_enabled_uc4e46e1"
# subject = "cpython.test_pyexpat.ReparseDeferralTest.test_reparse_deferral_enabled"
# kind = "semantic"
# xfail = "auto-extracted CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_pyexpat.py"
# status = "filled"
# ///
# mamba-xfail: auto-extracted CPython test; mamba promotion pending
import unittest, io
from test import test_pyexpat
_suite = unittest.defaultTestLoader.loadTestsFromName("ReparseDeferralTest.test_reparse_deferral_enabled", test_pyexpat)
_result = unittest.TextTestRunner(stream=io.StringIO(), verbosity=0).run(_suite)
assert _result.wasSuccessful(), "CPython ReparseDeferralTest.test_reparse_deferral_enabled did not pass"
print("ReparseDeferralTest::test_reparse_deferral_enabled: ok")
