# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "shelve"
# dimension = "behavior"
# case = "test_case__test_open_template_ucec0002"
# subject = "cpython.test_shelve.TestCase.test_open_template"
# kind = "semantic"
# xfail = "auto-extracted CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_shelve.py"
# status = "filled"
# ///
# mamba-xfail: auto-extracted CPython test; mamba promotion pending
import unittest, io
from test import test_shelve
_suite = unittest.defaultTestLoader.loadTestsFromName("TestCase.test_open_template", test_shelve)
_result = unittest.TextTestRunner(stream=io.StringIO(), verbosity=0).run(_suite)
assert _result.wasSuccessful(), "CPython TestCase.test_open_template did not pass"
print("TestCase::test_open_template: ok")
