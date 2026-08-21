# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "xml_etree_c"
# dimension = "behavior"
# case = "misc_tests__test_length_overflow_uc02d17d"
# subject = "cpython.test_xml_etree_c.MiscTests.test_length_overflow"
# kind = "semantic"
# xfail = "auto-extracted CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_xml_etree_c.py"
# status = "filled"
# ///
# mamba-xfail: auto-extracted CPython test; mamba promotion pending
import unittest, io
from test import test_xml_etree_c
_suite = unittest.defaultTestLoader.loadTestsFromName("MiscTests.test_length_overflow", test_xml_etree_c)
_result = unittest.TextTestRunner(stream=io.StringIO(), verbosity=0).run(_suite)
assert _result.wasSuccessful(), "CPython MiscTests.test_length_overflow did not pass"
print("MiscTests::test_length_overflow: ok")
