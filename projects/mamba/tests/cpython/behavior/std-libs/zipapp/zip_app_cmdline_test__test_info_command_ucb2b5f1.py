# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "zipapp"
# dimension = "behavior"
# case = "zip_app_cmdline_test__test_info_command_ucb2b5f1"
# subject = "cpython.test_zipapp.ZipAppCmdlineTest.test_info_command"
# kind = "semantic"
# xfail = "auto-extracted CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_zipapp.py"
# status = "filled"
# ///
# mamba-xfail: auto-extracted CPython test; mamba promotion pending
import unittest, io
from test import test_zipapp
_suite = unittest.defaultTestLoader.loadTestsFromName("ZipAppCmdlineTest.test_info_command", test_zipapp)
_result = unittest.TextTestRunner(stream=io.StringIO(), verbosity=0).run(_suite)
assert _result.wasSuccessful(), "CPython ZipAppCmdlineTest.test_info_command did not pass"
print("ZipAppCmdlineTest::test_info_command: ok")
