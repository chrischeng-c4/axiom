# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "pyclbr"
# dimension = "behavior"
# case = "pyclbr_test__test_nested_uc127235"
# subject = "cpython.test_pyclbr.PyclbrTest.test_nested"
# kind = "semantic"
# xfail = "auto-extracted CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_pyclbr.py"
# status = "filled"
# ///
# mamba-xfail: auto-extracted CPython test; mamba promotion pending
import unittest, io
from test import test_pyclbr
_suite = unittest.defaultTestLoader.loadTestsFromName("PyclbrTest.test_nested", test_pyclbr)
_result = unittest.TextTestRunner(stream=io.StringIO(), verbosity=0).run(_suite)
assert _result.wasSuccessful(), "CPython PyclbrTest.test_nested did not pass"
print("PyclbrTest::test_nested: ok")
