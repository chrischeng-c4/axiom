# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "compatibilty_files"
# dimension = "behavior"
# case = "compatibility_files_tests__test_spec_path_open_uc467857"
# subject = "cpython.test_compatibilty_files.CompatibilityFilesTests.test_spec_path_open"
# kind = "semantic"
# xfail = "auto-extracted CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_importlib/resources/test_compatibilty_files.py"
# status = "filled"
# ///
# mamba-xfail: auto-extracted CPython test; mamba promotion pending
import unittest, io
from test.test_importlib.resources import test_compatibilty_files
_suite = unittest.defaultTestLoader.loadTestsFromName("CompatibilityFilesTests.test_spec_path_open", test_compatibilty_files)
_result = unittest.TextTestRunner(stream=io.StringIO(), verbosity=0).run(_suite)
assert _result.wasSuccessful(), "CPython CompatibilityFilesTests.test_spec_path_open did not pass"
print("CompatibilityFilesTests::test_spec_path_open: ok")
