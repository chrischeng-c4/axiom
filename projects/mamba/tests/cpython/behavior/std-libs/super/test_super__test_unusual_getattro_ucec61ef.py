# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "super"
# dimension = "behavior"
# case = "test_super__test_unusual_getattro_ucec61ef"
# subject = "cpython.test_super.TestSuper.test_unusual_getattro"
# kind = "semantic"
# xfail = "auto-extracted CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_super.py"
# status = "filled"
# ///
# mamba-xfail: auto-extracted CPython test; mamba promotion pending
import unittest, io
from test import test_super
_suite = unittest.defaultTestLoader.loadTestsFromName("TestSuper.test_unusual_getattro", test_super)
_result = unittest.TextTestRunner(stream=io.StringIO(), verbosity=0).run(_suite)
assert _result.wasSuccessful(), "CPython TestSuper.test_unusual_getattro did not pass"
print("TestSuper::test_unusual_getattro: ok")
