# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "os"
# dimension = "behavior"
# case = "chown_file_tests__test_chown_uid_gid_arguments_must_be_index"
# subject = "cpython.test_os.ChownFileTests.test_chown_uid_gid_arguments_must_be_index"
# kind = "semantic"
# xfail = "auto-extracted CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_os.py"
# status = "filled"
# ///
# mamba-xfail: auto-extracted CPython test; mamba promotion pending
import unittest, io
from test import test_os
_suite = unittest.defaultTestLoader.loadTestsFromName("ChownFileTests.test_chown_uid_gid_arguments_must_be_index", test_os)
_result = unittest.TextTestRunner(stream=io.StringIO(), verbosity=0).run(_suite)
assert _result.wasSuccessful(), "CPython ChownFileTests.test_chown_uid_gid_arguments_must_be_index did not pass"
print("ChownFileTests::test_chown_uid_gid_arguments_must_be_index: ok")
