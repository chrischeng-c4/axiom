# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "pickle"
# dimension = "behavior"
# case = "c_pickler_unpickler_object_tests__test_issue18339_uc034136"
# subject = "cpython.test_pickle.CPicklerUnpicklerObjectTests.test_issue18339"
# kind = "semantic"
# xfail = "auto-extracted CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_pickle.py"
# status = "filled"
# ///
# mamba-xfail: auto-extracted CPython test; mamba promotion pending
import unittest, io
from test import test_pickle
_suite = unittest.defaultTestLoader.loadTestsFromName("CPicklerUnpicklerObjectTests.test_issue18339", test_pickle)
_result = unittest.TextTestRunner(stream=io.StringIO(), verbosity=0).run(_suite)
assert _result.wasSuccessful(), "CPython CPicklerUnpicklerObjectTests.test_issue18339 did not pass"
print("CPicklerUnpicklerObjectTests::test_issue18339: ok")
