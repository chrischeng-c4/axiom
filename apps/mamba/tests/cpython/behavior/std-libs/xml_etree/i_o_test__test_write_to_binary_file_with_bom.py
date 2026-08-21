# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "xml_etree"
# dimension = "behavior"
# case = "i_o_test__test_write_to_binary_file_with_bom"
# subject = "cpython.test_xml_etree.IOTest.test_write_to_binary_file_with_bom"
# kind = "semantic"
# xfail = "auto-extracted CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_xml_etree.py"
# status = "filled"
# ///
# mamba-xfail: auto-extracted CPython test; mamba promotion pending
import unittest, io
from test import test_xml_etree
_suite = unittest.defaultTestLoader.loadTestsFromName("IOTest.test_write_to_binary_file_with_bom", test_xml_etree)
_result = unittest.TextTestRunner(stream=io.StringIO(), verbosity=0).run(_suite)
assert _result.wasSuccessful(), "CPython IOTest.test_write_to_binary_file_with_bom did not pass"
print("IOTest::test_write_to_binary_file_with_bom: ok")
