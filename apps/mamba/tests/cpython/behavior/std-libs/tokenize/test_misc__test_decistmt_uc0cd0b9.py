# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "tokenize"
# dimension = "behavior"
# case = "test_misc__test_decistmt_uc0cd0b9"
# subject = "cpython.test_tokenize.TestMisc.test_decistmt"
# kind = "semantic"
# xfail = "auto-extracted CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_tokenize.py"
# status = "filled"
# ///
# mamba-xfail: auto-extracted CPython test; mamba promotion pending
import unittest, io
from test import test_tokenize
_suite = unittest.defaultTestLoader.loadTestsFromName("TestMisc.test_decistmt", test_tokenize)
_result = unittest.TextTestRunner(stream=io.StringIO(), verbosity=0).run(_suite)
assert _result.wasSuccessful(), "CPython TestMisc.test_decistmt did not pass"
print("TestMisc::test_decistmt: ok")
