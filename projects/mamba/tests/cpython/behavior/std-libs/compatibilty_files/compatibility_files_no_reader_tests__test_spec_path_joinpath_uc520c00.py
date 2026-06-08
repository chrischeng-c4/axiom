# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "compatibilty_files"
# dimension = "behavior"
# case = "compatibility_files_no_reader_tests__test_spec_path_joinpath_uc520c00"
# subject = "cpython.test_compatibilty_files.CompatibilityFilesNoReaderTests.test_spec_path_joinpath"
# kind = "semantic"
# xfail = "auto-extracted CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_importlib/resources/test_compatibilty_files.py"
# status = "filled"
# ///
# mamba-xfail: auto-extracted CPython test; mamba promotion pending
import unittest, io
from test.test_importlib.resources import test_compatibilty_files
_suite = unittest.defaultTestLoader.loadTestsFromName("CompatibilityFilesNoReaderTests.test_spec_path_joinpath", test_compatibilty_files)
_result = unittest.TextTestRunner(stream=io.StringIO(), verbosity=0).run(_suite)
assert _result.wasSuccessful(), "CPython CompatibilityFilesNoReaderTests.test_spec_path_joinpath did not pass"
print("CompatibilityFilesNoReaderTests::test_spec_path_joinpath: ok")
