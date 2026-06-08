# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "locale"
# dimension = "behavior"
# case = "test_en_u_s_collation__test_strxfrm_with_diacritic_uc6ac053"
# subject = "cpython.test_locale.TestEnUSCollation.test_strxfrm_with_diacritic"
# kind = "semantic"
# xfail = "auto-extracted CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_locale.py"
# status = "filled"
# ///
# mamba-xfail: auto-extracted CPython test; mamba promotion pending
import unittest, io
from test import test_locale
_suite = unittest.defaultTestLoader.loadTestsFromName("TestEnUSCollation.test_strxfrm_with_diacritic", test_locale)
_result = unittest.TextTestRunner(stream=io.StringIO(), verbosity=0).run(_suite)
assert _result.wasSuccessful(), "CPython TestEnUSCollation.test_strxfrm_with_diacritic did not pass"
print("TestEnUSCollation::test_strxfrm_with_diacritic: ok")
