# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "pickle"
# dimension = "behavior"
# case = "py_id_pers_pickler_tests__test_custom_pickler_dispatch_table_memleak_uc79da47"
# subject = "cpython.test_pickle.PyIdPersPicklerTests.test_custom_pickler_dispatch_table_memleak"
# kind = "semantic"
# xfail = "auto-extracted CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_pickle.py"
# status = "filled"
# ///
# mamba-xfail: auto-extracted CPython test; mamba promotion pending
import unittest, io
from test import test_pickle
_suite = unittest.defaultTestLoader.loadTestsFromName("PyIdPersPicklerTests.test_custom_pickler_dispatch_table_memleak", test_pickle)
_result = unittest.TextTestRunner(stream=io.StringIO(), verbosity=0).run(_suite)
assert _result.wasSuccessful(), "CPython PyIdPersPicklerTests.test_custom_pickler_dispatch_table_memleak did not pass"
print("PyIdPersPicklerTests::test_custom_pickler_dispatch_table_memleak: ok")
