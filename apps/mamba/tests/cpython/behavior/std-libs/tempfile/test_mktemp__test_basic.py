# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "tempfile"
# dimension = "behavior"
# case = "test_mktemp__test_basic"
# subject = "cpython.test_tempfile.TestMktemp.test_basic"
# kind = "semantic"
# xfail = "auto-extracted CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_tempfile.py"
# status = "filled"
# ///
# mamba-xfail: auto-extracted CPython test; mamba promotion pending
import unittest, io
from test import test_tempfile
_suite = unittest.defaultTestLoader.loadTestsFromName("TestMktemp.test_basic", test_tempfile)
_result = unittest.TextTestRunner(stream=io.StringIO(), verbosity=0).run(_suite)
assert _result.wasSuccessful(), "CPython TestMktemp.test_basic did not pass"
print("TestMktemp::test_basic: ok")
