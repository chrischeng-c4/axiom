# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "xml_etree"
# dimension = "behavior"
# case = "no_accelerator_test__test_correct_import_pyET"
# subject = "cpython.test_xml_etree.NoAcceleratorTest.test_correct_import_pyET"
# kind = "semantic"
# xfail = "auto-extracted CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_xml_etree.py"
# status = "filled"
# ///
# mamba-xfail: auto-extracted CPython test; mamba promotion pending
import unittest, io
from test import test_xml_etree
_suite = unittest.defaultTestLoader.loadTestsFromName("NoAcceleratorTest.test_correct_import_pyET", test_xml_etree)
_result = unittest.TextTestRunner(stream=io.StringIO(), verbosity=0).run(_suite)
assert _result.wasSuccessful(), "CPython NoAcceleratorTest.test_correct_import_pyET did not pass"
print("NoAcceleratorTest::test_correct_import_pyET: ok")
