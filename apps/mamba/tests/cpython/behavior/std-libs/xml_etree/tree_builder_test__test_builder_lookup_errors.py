# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "xml_etree"
# dimension = "behavior"
# case = "tree_builder_test__test_builder_lookup_errors"
# subject = "cpython.test_xml_etree.TreeBuilderTest.test_builder_lookup_errors"
# kind = "semantic"
# xfail = "auto-extracted CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_xml_etree.py"
# status = "filled"
# ///
# mamba-xfail: auto-extracted CPython test; mamba promotion pending
import unittest, io
from test import test_xml_etree
_suite = unittest.defaultTestLoader.loadTestsFromName("TreeBuilderTest.test_builder_lookup_errors", test_xml_etree)
_result = unittest.TextTestRunner(stream=io.StringIO(), verbosity=0).run(_suite)
assert _result.wasSuccessful(), "CPython TreeBuilderTest.test_builder_lookup_errors did not pass"
print("TreeBuilderTest::test_builder_lookup_errors: ok")
