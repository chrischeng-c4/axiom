# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "webbrowser"
# dimension = "behavior"
# case = "import_test__test_environment_ucf3111c"
# subject = "cpython.test_webbrowser.ImportTest.test_environment"
# kind = "semantic"
# xfail = "auto-extracted CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_webbrowser.py"
# status = "filled"
# ///
# mamba-xfail: auto-extracted CPython test; mamba promotion pending
import unittest, io
from test import test_webbrowser
_suite = unittest.defaultTestLoader.loadTestsFromName("ImportTest.test_environment", test_webbrowser)
_result = unittest.TextTestRunner(stream=io.StringIO(), verbosity=0).run(_suite)
assert _result.wasSuccessful(), "CPython ImportTest.test_environment did not pass"
print("ImportTest::test_environment: ok")
