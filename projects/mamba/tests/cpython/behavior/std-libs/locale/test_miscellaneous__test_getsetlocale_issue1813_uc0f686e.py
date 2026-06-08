# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "locale"
# dimension = "behavior"
# case = "test_miscellaneous__test_getsetlocale_issue1813_uc0f686e"
# subject = "cpython.test_locale.TestMiscellaneous.test_getsetlocale_issue1813"
# kind = "semantic"
# xfail = "auto-extracted CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_locale.py"
# status = "filled"
# ///
# mamba-xfail: auto-extracted CPython test; mamba promotion pending
import unittest, io
from test import test_locale
_suite = unittest.defaultTestLoader.loadTestsFromName("TestMiscellaneous.test_getsetlocale_issue1813", test_locale)
_result = unittest.TextTestRunner(stream=io.StringIO(), verbosity=0).run(_suite)
assert _result.wasSuccessful(), "CPython TestMiscellaneous.test_getsetlocale_issue1813 did not pass"
print("TestMiscellaneous::test_getsetlocale_issue1813: ok")
