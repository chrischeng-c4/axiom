# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "tempfile"
# dimension = "behavior"
# case = "test_mkdtemp__test_collision_with_existing_directory"
# subject = "cpython.test_tempfile.TestMkdtemp.test_collision_with_existing_directory"
# kind = "semantic"
# xfail = "auto-extracted CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_tempfile.py"
# status = "filled"
# ///
# mamba-xfail: auto-extracted CPython test; mamba promotion pending
import unittest, io
from test import test_tempfile
_suite = unittest.defaultTestLoader.loadTestsFromName("TestMkdtemp.test_collision_with_existing_directory", test_tempfile)
_result = unittest.TextTestRunner(stream=io.StringIO(), verbosity=0).run(_suite)
assert _result.wasSuccessful(), "CPython TestMkdtemp.test_collision_with_existing_directory did not pass"
print("TestMkdtemp::test_collision_with_existing_directory: ok")
