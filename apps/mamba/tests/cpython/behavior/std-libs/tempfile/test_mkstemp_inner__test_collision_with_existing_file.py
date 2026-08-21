# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "tempfile"
# dimension = "behavior"
# case = "test_mkstemp_inner__test_collision_with_existing_file"
# subject = "cpython.test_tempfile.TestMkstempInner.test_collision_with_existing_file"
# kind = "semantic"
# xfail = "auto-extracted CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_tempfile.py"
# status = "filled"
# ///
# mamba-xfail: auto-extracted CPython test; mamba promotion pending
import unittest, io
from test import test_tempfile
_suite = unittest.defaultTestLoader.loadTestsFromName("TestMkstempInner.test_collision_with_existing_file", test_tempfile)
_result = unittest.TextTestRunner(stream=io.StringIO(), verbosity=0).run(_suite)
assert _result.wasSuccessful(), "CPython TestMkstempInner.test_collision_with_existing_file did not pass"
print("TestMkstempInner::test_collision_with_existing_file: ok")
