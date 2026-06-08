# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "os"
# dimension = "behavior"
# case = "stat_attribute_tests__test_stat_attributes_bytes"
# subject = "cpython.test_os.StatAttributeTests.test_stat_attributes_bytes"
# kind = "semantic"
# xfail = "auto-extracted CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_os.py"
# status = "filled"
# ///
# mamba-xfail: auto-extracted CPython test; mamba promotion pending
import unittest, io
from test import test_os
_suite = unittest.defaultTestLoader.loadTestsFromName("StatAttributeTests.test_stat_attributes_bytes", test_os)
_result = unittest.TextTestRunner(stream=io.StringIO(), verbosity=0).run(_suite)
assert _result.wasSuccessful(), "CPython StatAttributeTests.test_stat_attributes_bytes did not pass"
print("StatAttributeTests::test_stat_attributes_bytes: ok")
