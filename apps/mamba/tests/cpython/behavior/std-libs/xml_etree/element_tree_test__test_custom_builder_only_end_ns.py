# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "xml_etree"
# dimension = "behavior"
# case = "element_tree_test__test_custom_builder_only_end_ns"
# subject = "cpython.test_xml_etree.ElementTreeTest.test_custom_builder_only_end_ns"
# kind = "semantic"
# xfail = "auto-extracted CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_xml_etree.py"
# status = "filled"
# ///
# mamba-xfail: auto-extracted CPython test; mamba promotion pending
import unittest, io
from test import test_xml_etree
_suite = unittest.defaultTestLoader.loadTestsFromName("ElementTreeTest.test_custom_builder_only_end_ns", test_xml_etree)
_result = unittest.TextTestRunner(stream=io.StringIO(), verbosity=0).run(_suite)
assert _result.wasSuccessful(), "CPython ElementTreeTest.test_custom_builder_only_end_ns did not pass"
print("ElementTreeTest::test_custom_builder_only_end_ns: ok")
