# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "xml_etree_c"
# dimension = "behavior"
# case = "sizeof_test__test_element_with_children_uc6c0275"
# subject = "cpython.test_xml_etree_c.SizeofTest.test_element_with_children"
# kind = "semantic"
# xfail = "auto-extracted CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_xml_etree_c.py"
# status = "filled"
# ///
# mamba-xfail: auto-extracted CPython test; mamba promotion pending
import unittest, io
from test import test_xml_etree_c
_suite = unittest.defaultTestLoader.loadTestsFromName("SizeofTest.test_element_with_children", test_xml_etree_c)
_result = unittest.TextTestRunner(stream=io.StringIO(), verbosity=0).run(_suite)
assert _result.wasSuccessful(), "CPython SizeofTest.test_element_with_children did not pass"
print("SizeofTest::test_element_with_children: ok")
