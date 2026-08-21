# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "format"
# dimension = "behavior"
# case = "format_test__test_locale_uce13e6b"
# subject = "cpython.test_format.FormatTest.test_locale"
# kind = "semantic"
# xfail = "auto-extracted CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_format.py"
# status = "filled"
# ///
# mamba-xfail: auto-extracted CPython test; mamba promotion pending
import unittest, io
from test import test_format
_suite = unittest.defaultTestLoader.loadTestsFromName("FormatTest.test_locale", test_format)
_result = unittest.TextTestRunner(stream=io.StringIO(), verbosity=0).run(_suite)
assert _result.wasSuccessful(), "CPython FormatTest.test_locale did not pass"
print("FormatTest::test_locale: ok")
