# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "builtin-libs"
# lib = "enumerate"
# dimension = "behavior"
# case = "enumerate_test_case__test_illformediterable_ucb03a9a"
# subject = "cpython.test_enumerate.EnumerateTestCase.test_illformediterable"
# kind = "semantic"
# xfail = "auto-extracted CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_enumerate.py"
# status = "filled"
# ///
# mamba-xfail: auto-extracted CPython test; mamba promotion pending
import unittest, io
from test import test_enumerate
_suite = unittest.defaultTestLoader.loadTestsFromName("EnumerateTestCase.test_illformediterable", test_enumerate)
_result = unittest.TextTestRunner(stream=io.StringIO(), verbosity=0).run(_suite)
assert _result.wasSuccessful(), "CPython EnumerateTestCase.test_illformediterable did not pass"
print("EnumerateTestCase::test_illformediterable: ok")
