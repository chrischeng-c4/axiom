# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "getpass"
# dimension = "behavior"
# case = "getpass_rawinput_test__test_uses_stdin_as_default_input_ucf5f0a1"
# subject = "cpython.test_getpass.GetpassRawinputTest.test_uses_stdin_as_default_input"
# kind = "semantic"
# xfail = "auto-extracted CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_getpass.py"
# status = "filled"
# ///
# mamba-xfail: auto-extracted CPython test; mamba promotion pending
import unittest, io
from test import test_getpass
_suite = unittest.defaultTestLoader.loadTestsFromName("GetpassRawinputTest.test_uses_stdin_as_default_input", test_getpass)
_result = unittest.TextTestRunner(stream=io.StringIO(), verbosity=0).run(_suite)
assert _result.wasSuccessful(), "CPython GetpassRawinputTest.test_uses_stdin_as_default_input did not pass"
print("GetpassRawinputTest::test_uses_stdin_as_default_input: ok")
