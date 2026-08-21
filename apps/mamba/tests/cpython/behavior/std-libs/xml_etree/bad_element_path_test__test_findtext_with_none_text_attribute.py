# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "xml_etree"
# dimension = "behavior"
# case = "bad_element_path_test__test_findtext_with_none_text_attribute"
# subject = "cpython.test_xml_etree.BadElementPathTest.test_findtext_with_none_text_attribute"
# kind = "semantic"
# xfail = "auto-extracted CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_xml_etree.py"
# status = "filled"
# ///
# mamba-xfail: auto-extracted CPython test; mamba promotion pending
import unittest, io
from test import test_xml_etree
_suite = unittest.defaultTestLoader.loadTestsFromName("BadElementPathTest.test_findtext_with_none_text_attribute", test_xml_etree)
_result = unittest.TextTestRunner(stream=io.StringIO(), verbosity=0).run(_suite)
assert _result.wasSuccessful(), "CPython BadElementPathTest.test_findtext_with_none_text_attribute did not pass"
print("BadElementPathTest::test_findtext_with_none_text_attribute: ok")
