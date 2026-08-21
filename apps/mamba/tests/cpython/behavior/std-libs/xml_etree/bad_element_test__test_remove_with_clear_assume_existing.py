# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "xml_etree"
# dimension = "behavior"
# case = "bad_element_test__test_remove_with_clear_assume_existing"
# subject = "cpython.test_xml_etree.BadElementTest.test_remove_with_clear_assume_existing"
# kind = "semantic"
# xfail = "auto-extracted CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_xml_etree.py"
# status = "filled"
# ///
# mamba-xfail: auto-extracted CPython test; mamba promotion pending
import unittest, io
from test import test_xml_etree
_suite = unittest.defaultTestLoader.loadTestsFromName("BadElementTest.test_remove_with_clear_assume_existing", test_xml_etree)
_result = unittest.TextTestRunner(stream=io.StringIO(), verbosity=0).run(_suite)
assert _result.wasSuccessful(), "CPython BadElementTest.test_remove_with_clear_assume_existing did not pass"
print("BadElementTest::test_remove_with_clear_assume_existing: ok")
