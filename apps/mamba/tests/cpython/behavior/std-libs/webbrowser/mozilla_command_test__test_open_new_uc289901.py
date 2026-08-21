# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "webbrowser"
# dimension = "behavior"
# case = "mozilla_command_test__test_open_new_uc289901"
# subject = "cpython.test_webbrowser.MozillaCommandTest.test_open_new"
# kind = "semantic"
# xfail = "auto-extracted CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_webbrowser.py"
# status = "filled"
# ///
# mamba-xfail: auto-extracted CPython test; mamba promotion pending
import unittest, io
from test import test_webbrowser
_suite = unittest.defaultTestLoader.loadTestsFromName("MozillaCommandTest.test_open_new", test_webbrowser)
_result = unittest.TextTestRunner(stream=io.StringIO(), verbosity=0).run(_suite)
assert _result.wasSuccessful(), "CPython MozillaCommandTest.test_open_new did not pass"
print("MozillaCommandTest::test_open_new: ok")
