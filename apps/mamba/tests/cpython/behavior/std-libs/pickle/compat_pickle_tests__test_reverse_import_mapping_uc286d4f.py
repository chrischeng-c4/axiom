# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "pickle"
# dimension = "behavior"
# case = "compat_pickle_tests__test_reverse_import_mapping_uc286d4f"
# subject = "cpython.test_pickle.CompatPickleTests.test_reverse_import_mapping"
# kind = "semantic"
# xfail = "auto-extracted CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_pickle.py"
# status = "filled"
# ///
# mamba-xfail: auto-extracted CPython test; mamba promotion pending
import unittest, io
from test import test_pickle
_suite = unittest.defaultTestLoader.loadTestsFromName("CompatPickleTests.test_reverse_import_mapping", test_pickle)
_result = unittest.TextTestRunner(stream=io.StringIO(), verbosity=0).run(_suite)
assert _result.wasSuccessful(), "CPython CompatPickleTests.test_reverse_import_mapping did not pass"
print("CompatPickleTests::test_reverse_import_mapping: ok")
