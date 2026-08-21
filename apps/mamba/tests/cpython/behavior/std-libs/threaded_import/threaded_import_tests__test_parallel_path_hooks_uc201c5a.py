# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "threaded_import"
# dimension = "behavior"
# case = "threaded_import_tests__test_parallel_path_hooks_uc201c5a"
# subject = "cpython.test_threaded_import.ThreadedImportTests.test_parallel_path_hooks"
# kind = "semantic"
# xfail = "auto-extracted CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_importlib/test_threaded_import.py"
# status = "filled"
# ///
# mamba-xfail: auto-extracted CPython test; mamba promotion pending
import unittest, io
from test.test_importlib import test_threaded_import
_suite = unittest.defaultTestLoader.loadTestsFromName("ThreadedImportTests.test_parallel_path_hooks", test_threaded_import)
_result = unittest.TextTestRunner(stream=io.StringIO(), verbosity=0).run(_suite)
assert _result.wasSuccessful(), "CPython ThreadedImportTests.test_parallel_path_hooks did not pass"
print("ThreadedImportTests::test_parallel_path_hooks: ok")
