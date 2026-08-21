# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "webbrowser"
# dimension = "behavior"
# case = "chrome_command_test__test_open_new_tab_uc1d2fa2"
# subject = "cpython.test_webbrowser.ChromeCommandTest.test_open_new_tab"
# kind = "semantic"
# xfail = "auto-extracted CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_webbrowser.py"
# status = "filled"
# ///
# mamba-xfail: auto-extracted CPython test; mamba promotion pending
import unittest, io
from test import test_webbrowser
_suite = unittest.defaultTestLoader.loadTestsFromName("ChromeCommandTest.test_open_new_tab", test_webbrowser)
_result = unittest.TextTestRunner(stream=io.StringIO(), verbosity=0).run(_suite)
assert _result.wasSuccessful(), "CPython ChromeCommandTest.test_open_new_tab did not pass"
print("ChromeCommandTest::test_open_new_tab: ok")
