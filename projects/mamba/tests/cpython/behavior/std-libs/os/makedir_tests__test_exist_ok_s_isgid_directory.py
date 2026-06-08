# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "os"
# dimension = "behavior"
# case = "makedir_tests__test_exist_ok_s_isgid_directory"
# subject = "cpython.test_os.MakedirTests.test_exist_ok_s_isgid_directory"
# kind = "semantic"
# xfail = "auto-extracted CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_os.py"
# status = "filled"
# ///
# mamba-xfail: auto-extracted CPython test; mamba promotion pending
import unittest, io
from test import test_os
_suite = unittest.defaultTestLoader.loadTestsFromName("MakedirTests.test_exist_ok_s_isgid_directory", test_os)
_result = unittest.TextTestRunner(stream=io.StringIO(), verbosity=0).run(_suite)
assert _result.wasSuccessful(), "CPython MakedirTests.test_exist_ok_s_isgid_directory did not pass"
print("MakedirTests::test_exist_ok_s_isgid_directory: ok")
