# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "xml_etree"
# dimension = "behavior"
# case = "namespace_parse_test__test_find_with_namespace"
# subject = "cpython.test_xml_etree.NamespaceParseTest.test_find_with_namespace"
# kind = "semantic"
# xfail = "auto-extracted CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_xml_etree.py"
# status = "filled"
# ///
# mamba-xfail: auto-extracted CPython test; mamba promotion pending
import unittest, io
from test import test_xml_etree
_suite = unittest.defaultTestLoader.loadTestsFromName("NamespaceParseTest.test_find_with_namespace", test_xml_etree)
_result = unittest.TextTestRunner(stream=io.StringIO(), verbosity=0).run(_suite)
assert _result.wasSuccessful(), "CPython NamespaceParseTest.test_find_with_namespace did not pass"
print("NamespaceParseTest::test_find_with_namespace: ok")
