# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "getpass"
# dimension = "behavior"
# case = "getpass_getuser_test__test_username_takes_username_from_env_uc1cfd9d"
# subject = "cpython.test_getpass.GetpassGetuserTest.test_username_takes_username_from_env"
# kind = "semantic"
# xfail = "auto-extracted CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_getpass.py"
# status = "filled"
# ///
# mamba-xfail: auto-extracted CPython test; mamba promotion pending
import unittest, io
from test import test_getpass
_suite = unittest.defaultTestLoader.loadTestsFromName("GetpassGetuserTest.test_username_takes_username_from_env", test_getpass)
_result = unittest.TextTestRunner(stream=io.StringIO(), verbosity=0).run(_suite)
assert _result.wasSuccessful(), "CPython GetpassGetuserTest.test_username_takes_username_from_env did not pass"
print("GetpassGetuserTest::test_username_takes_username_from_env: ok")
