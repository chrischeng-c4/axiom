# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "pickle"
# dimension = "behavior"
# case = "compat_pickle_tests__test_exceptions_uc0f80a5"
# subject = "cpython.test_pickle.CompatPickleTests.test_exceptions"
# kind = "semantic"
# xfail = "auto-extracted CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_pickle.py"
# status = "filled"
# ///
# mamba-xfail: auto-extracted CPython test; mamba promotion pending
import unittest, io
from test import test_pickle
_suite = unittest.defaultTestLoader.loadTestsFromName("CompatPickleTests.test_exceptions", test_pickle)
_result = unittest.TextTestRunner(stream=io.StringIO(), verbosity=0).run(_suite)
assert _result.wasSuccessful(), "CPython CompatPickleTests.test_exceptions did not pass"
print("CompatPickleTests::test_exceptions: ok")
