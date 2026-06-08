# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "gettext"
# dimension = "behavior"
# case = "g_n_u_translation_parsing_test__test_ignore_comments_in_headers_issue36239"
# subject = "cpython.test_gettext.GNUTranslationParsingTest.test_ignore_comments_in_headers_issue36239"
# kind = "semantic"
# xfail = "auto-extracted CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_gettext.py"
# status = "filled"
# ///
# mamba-xfail: auto-extracted CPython test; mamba promotion pending
import unittest, io
from test import test_gettext
_suite = unittest.defaultTestLoader.loadTestsFromName("GNUTranslationParsingTest.test_ignore_comments_in_headers_issue36239", test_gettext)
_result = unittest.TextTestRunner(stream=io.StringIO(), verbosity=0).run(_suite)
assert _result.wasSuccessful(), "CPython GNUTranslationParsingTest.test_ignore_comments_in_headers_issue36239 did not pass"
print("GNUTranslationParsingTest::test_ignore_comments_in_headers_issue36239: ok")
