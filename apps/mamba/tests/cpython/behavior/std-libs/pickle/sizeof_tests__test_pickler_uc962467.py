# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "pickle"
# dimension = "behavior"
# case = "sizeof_tests__test_pickler_uc962467"
# subject = "cpython.test_pickle.SizeofTests.test_pickler"
# kind = "semantic"
# xfail = "auto-extracted CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_pickle.py"
# status = "filled"
# ///
# mamba-xfail: auto-extracted CPython test; mamba promotion pending
import unittest, io
from test import test_pickle
_suite = unittest.defaultTestLoader.loadTestsFromName("SizeofTests.test_pickler", test_pickle)
_result = unittest.TextTestRunner(stream=io.StringIO(), verbosity=0).run(_suite)
assert _result.wasSuccessful(), "CPython SizeofTests.test_pickler did not pass"
print("SizeofTests::test_pickler: ok")
