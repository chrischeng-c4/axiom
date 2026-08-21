# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "path"
# dimension = "behavior"
# case = "path_zip_tests__test_remove_in_context_manager"
# subject = "cpython.test_path.PathZipTests.test_remove_in_context_manager"
# kind = "semantic"
# xfail = "auto-extracted CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_importlib/resources/test_path.py"
# status = "filled"
# ///
# mamba-xfail: auto-extracted CPython test; mamba promotion pending
import unittest, io
from test.test_importlib.resources import test_path
_suite = unittest.defaultTestLoader.loadTestsFromName("PathZipTests.test_remove_in_context_manager", test_path)
_result = unittest.TextTestRunner(stream=io.StringIO(), verbosity=0).run(_suite)
assert _result.wasSuccessful(), "CPython PathZipTests.test_remove_in_context_manager did not pass"
print("PathZipTests::test_remove_in_context_manager: ok")
