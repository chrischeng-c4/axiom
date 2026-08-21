# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "xml_etree"
# dimension = "behavior"
# case = "x_m_l_pull_parser_test__test_feed_while_iterating"
# subject = "cpython.test_xml_etree.XMLPullParserTest.test_feed_while_iterating"
# kind = "semantic"
# xfail = "auto-extracted CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_xml_etree.py"
# status = "filled"
# ///
# mamba-xfail: auto-extracted CPython test; mamba promotion pending
import unittest, io
from test import test_xml_etree
_suite = unittest.defaultTestLoader.loadTestsFromName("XMLPullParserTest.test_feed_while_iterating", test_xml_etree)
_result = unittest.TextTestRunner(stream=io.StringIO(), verbosity=0).run(_suite)
assert _result.wasSuccessful(), "CPython XMLPullParserTest.test_feed_while_iterating did not pass"
print("XMLPullParserTest::test_feed_while_iterating: ok")
