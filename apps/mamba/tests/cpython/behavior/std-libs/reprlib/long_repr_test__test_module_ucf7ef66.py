# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "reprlib"
# dimension = "behavior"
# case = "long_repr_test__test_module_ucf7ef66"
# subject = "cpython.test_reprlib.LongReprTest.test_module"
# kind = "semantic"
# xfail = "auto-extracted CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_reprlib.py"
# status = "filled"
# ///
# mamba-xfail: auto-extracted CPython test; mamba promotion pending
import unittest, io
from test import test_reprlib
_suite = unittest.defaultTestLoader.loadTestsFromName("LongReprTest.test_module", test_reprlib)
_result = unittest.TextTestRunner(stream=io.StringIO(), verbosity=0).run(_suite)
assert _result.wasSuccessful(), "CPython LongReprTest.test_module did not pass"
print("LongReprTest::test_module: ok")
