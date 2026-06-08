# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "builtin-libs"
# lib = "enumerate"
# dimension = "behavior"
# case = "test_reversed__test_bug1229429_uc9a8389"
# subject = "cpython.test_enumerate.TestReversed.test_bug1229429"
# kind = "semantic"
# xfail = "auto-extracted CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_enumerate.py"
# status = "filled"
# ///
# mamba-xfail: auto-extracted CPython test; mamba promotion pending
import unittest, io
from test import test_enumerate
_suite = unittest.defaultTestLoader.loadTestsFromName("TestReversed.test_bug1229429", test_enumerate)
_result = unittest.TextTestRunner(stream=io.StringIO(), verbosity=0).run(_suite)
assert _result.wasSuccessful(), "CPython TestReversed.test_bug1229429 did not pass"
print("TestReversed::test_bug1229429: ok")
