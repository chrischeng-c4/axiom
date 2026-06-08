# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "reprlib"
# dimension = "behavior"
# case = "long_repr_test__test_class_ucfa6999"
# subject = "cpython.test_reprlib.LongReprTest.test_class"
# kind = "semantic"
# xfail = "auto-extracted CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_reprlib.py"
# status = "filled"
# ///
# mamba-xfail: auto-extracted CPython test; mamba promotion pending
import unittest, io
from test import test_reprlib
_suite = unittest.defaultTestLoader.loadTestsFromName("LongReprTest.test_class", test_reprlib)
_result = unittest.TextTestRunner(stream=io.StringIO(), verbosity=0).run(_suite)
assert _result.wasSuccessful(), "CPython LongReprTest.test_class did not pass"
print("LongReprTest::test_class: ok")
