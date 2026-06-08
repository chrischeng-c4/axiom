# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "pyclbr"
# dimension = "behavior"
# case = "pyclbr_test__test_easy_uc72199f"
# subject = "cpython.test_pyclbr.PyclbrTest.test_easy"
# kind = "semantic"
# xfail = "auto-extracted CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_pyclbr.py"
# status = "filled"
# ///
# mamba-xfail: auto-extracted CPython test; mamba promotion pending
import unittest, io
from test import test_pyclbr
_suite = unittest.defaultTestLoader.loadTestsFromName("PyclbrTest.test_easy", test_pyclbr)
_result = unittest.TextTestRunner(stream=io.StringIO(), verbosity=0).run(_suite)
assert _result.wasSuccessful(), "CPython PyclbrTest.test_easy did not pass"
print("PyclbrTest::test_easy: ok")
