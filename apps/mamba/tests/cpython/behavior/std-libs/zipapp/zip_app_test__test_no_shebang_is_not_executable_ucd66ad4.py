# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "zipapp"
# dimension = "behavior"
# case = "zip_app_test__test_no_shebang_is_not_executable_ucd66ad4"
# subject = "cpython.test_zipapp.ZipAppTest.test_no_shebang_is_not_executable"
# kind = "semantic"
# xfail = "auto-extracted CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_zipapp.py"
# status = "filled"
# ///
# mamba-xfail: auto-extracted CPython test; mamba promotion pending
import unittest, io
from test import test_zipapp
_suite = unittest.defaultTestLoader.loadTestsFromName("ZipAppTest.test_no_shebang_is_not_executable", test_zipapp)
_result = unittest.TextTestRunner(stream=io.StringIO(), verbosity=0).run(_suite)
assert _result.wasSuccessful(), "CPython ZipAppTest.test_no_shebang_is_not_executable did not pass"
print("ZipAppTest::test_no_shebang_is_not_executable: ok")
