# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "sys"
# dimension = "behavior"
# case = "sizeof_test__test_objecttypes_ucf3cf8a"
# subject = "cpython.test_sys.SizeofTest.test_objecttypes"
# kind = "semantic"
# xfail = "auto-extracted CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_sys.py"
# status = "filled"
# ///
# mamba-xfail: auto-extracted CPython test; mamba promotion pending
import unittest, io
from test import test_sys
_suite = unittest.defaultTestLoader.loadTestsFromName("SizeofTest.test_objecttypes", test_sys)
_result = unittest.TextTestRunner(stream=io.StringIO(), verbosity=0).run(_suite)
assert _result.wasSuccessful(), "CPython SizeofTest.test_objecttypes did not pass"
print("SizeofTest::test_objecttypes: ok")
