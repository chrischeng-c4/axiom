# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "codecs"
# dimension = "behavior"
# case = "code_page_test__test_mbcs_alias"
# subject = "cpython.test_codecs.CodePageTest.test_mbcs_alias"
# kind = "semantic"
# xfail = "auto-extracted CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_codecs.py"
# status = "filled"
# ///
# mamba-xfail: auto-extracted CPython test; mamba promotion pending
import unittest, io
from test import test_codecs
_suite = unittest.defaultTestLoader.loadTestsFromName("CodePageTest.test_mbcs_alias", test_codecs)
_result = unittest.TextTestRunner(stream=io.StringIO(), verbosity=0).run(_suite)
assert _result.wasSuccessful(), "CPython CodePageTest.test_mbcs_alias did not pass"
print("CodePageTest::test_mbcs_alias: ok")
