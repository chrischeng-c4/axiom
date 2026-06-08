# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "htmlparser"
# dimension = "behavior"
# case = "h_t_m_l_parser_test_case__test_textarea_content_uc89eeb1"
# subject = "cpython.test_htmlparser.HTMLParserTestCase.test_textarea_content"
# kind = "semantic"
# xfail = "auto-extracted CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_htmlparser.py"
# status = "filled"
# ///
# mamba-xfail: auto-extracted CPython test; mamba promotion pending
import unittest, io
from test import test_htmlparser
_suite = unittest.defaultTestLoader.loadTestsFromName("HTMLParserTestCase.test_textarea_content", test_htmlparser)
_result = unittest.TextTestRunner(stream=io.StringIO(), verbosity=0).run(_suite)
assert _result.wasSuccessful(), "CPython HTMLParserTestCase.test_textarea_content did not pass"
print("HTMLParserTestCase::test_textarea_content: ok")
