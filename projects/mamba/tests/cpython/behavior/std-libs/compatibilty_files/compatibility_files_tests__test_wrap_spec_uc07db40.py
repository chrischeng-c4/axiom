# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "compatibilty_files"
# dimension = "behavior"
# case = "compatibility_files_tests__test_wrap_spec_uc07db40"
# subject = "cpython.test_compatibilty_files.CompatibilityFilesTests.test_wrap_spec"
# kind = "semantic"
# xfail = "auto-extracted CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_importlib/resources/test_compatibilty_files.py"
# status = "filled"
# ///
# mamba-xfail: auto-extracted CPython test; mamba promotion pending
import unittest, io
from test.test_importlib.resources import test_compatibilty_files
_suite = unittest.defaultTestLoader.loadTestsFromName("CompatibilityFilesTests.test_wrap_spec", test_compatibilty_files)
_result = unittest.TextTestRunner(stream=io.StringIO(), verbosity=0).run(_suite)
assert _result.wasSuccessful(), "CPython CompatibilityFilesTests.test_wrap_spec did not pass"
print("CompatibilityFilesTests::test_wrap_spec: ok")
