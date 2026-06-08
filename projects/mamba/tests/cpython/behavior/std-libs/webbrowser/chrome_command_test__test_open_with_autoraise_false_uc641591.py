# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "webbrowser"
# dimension = "behavior"
# case = "chrome_command_test__test_open_with_autoraise_false_uc641591"
# subject = "cpython.test_webbrowser.ChromeCommandTest.test_open_with_autoraise_false"
# kind = "semantic"
# xfail = "auto-extracted CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_webbrowser.py"
# status = "filled"
# ///
# mamba-xfail: auto-extracted CPython test; mamba promotion pending
import unittest, io
from test import test_webbrowser
_suite = unittest.defaultTestLoader.loadTestsFromName("ChromeCommandTest.test_open_with_autoraise_false", test_webbrowser)
_result = unittest.TextTestRunner(stream=io.StringIO(), verbosity=0).run(_suite)
assert _result.wasSuccessful(), "CPython ChromeCommandTest.test_open_with_autoraise_false did not pass"
print("ChromeCommandTest::test_open_with_autoraise_false: ok")
