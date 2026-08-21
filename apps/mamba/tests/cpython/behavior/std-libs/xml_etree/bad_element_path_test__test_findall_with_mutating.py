# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "xml_etree"
# dimension = "behavior"
# case = "bad_element_path_test__test_findall_with_mutating"
# subject = "cpython.test_xml_etree.BadElementPathTest.test_findall_with_mutating"
# kind = "semantic"
# xfail = "auto-extracted CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_xml_etree.py"
# status = "filled"
# ///
# mamba-xfail: auto-extracted CPython test; mamba promotion pending
import unittest, io
from test import test_xml_etree
_suite = unittest.defaultTestLoader.loadTestsFromName("BadElementPathTest.test_findall_with_mutating", test_xml_etree)
_result = unittest.TextTestRunner(stream=io.StringIO(), verbosity=0).run(_suite)
assert _result.wasSuccessful(), "CPython BadElementPathTest.test_findall_with_mutating did not pass"
print("BadElementPathTest::test_findall_with_mutating: ok")
