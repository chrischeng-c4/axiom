# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "core"
# dimension = "behavior"
# case = "zip_info_tests__test_from_file_pathlike"
# subject = "cpython.test_core.ZipInfoTests.test_from_file_pathlike"
# kind = "semantic"
# xfail = "auto-extracted CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_zipfile/test_core.py"
# status = "filled"
# ///
# mamba-xfail: auto-extracted CPython test; mamba promotion pending
import unittest, io
from test.test_zipfile import test_core
_suite = unittest.defaultTestLoader.loadTestsFromName("ZipInfoTests.test_from_file_pathlike", test_core)
_result = unittest.TextTestRunner(stream=io.StringIO(), verbosity=0).run(_suite)
assert _result.wasSuccessful(), "CPython ZipInfoTests.test_from_file_pathlike did not pass"
print("ZipInfoTests::test_from_file_pathlike: ok")
