# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "locale"
# dimension = "behavior"
# case = "test_en_u_s_number_formatting__test_currency_uccfe4fc"
# subject = "cpython.test_locale.TestEnUSNumberFormatting.test_currency"
# kind = "semantic"
# xfail = "auto-extracted CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_locale.py"
# status = "filled"
# ///
# mamba-xfail: auto-extracted CPython test; mamba promotion pending
import unittest, io
from test import test_locale
_suite = unittest.defaultTestLoader.loadTestsFromName("TestEnUSNumberFormatting.test_currency", test_locale)
_result = unittest.TextTestRunner(stream=io.StringIO(), verbosity=0).run(_suite)
assert _result.wasSuccessful(), "CPython TestEnUSNumberFormatting.test_currency did not pass"
print("TestEnUSNumberFormatting::test_currency: ok")
