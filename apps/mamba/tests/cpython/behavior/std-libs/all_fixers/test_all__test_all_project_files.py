# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "all_fixers"
# dimension = "behavior"
# case = "test_all__test_all_project_files"
# subject = "cpython.test_all_fixers.Test_all.test_all_project_files"
# kind = "semantic"
# xfail = "auto-extracted CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_lib2to3/test_all_fixers.py"
# status = "filled"
# ///
# mamba-xfail: auto-extracted CPython test; mamba promotion pending
import unittest, io
from test.test_lib2to3 import test_all_fixers
_suite = unittest.defaultTestLoader.loadTestsFromName("Test_all.test_all_project_files", test_all_fixers)
_result = unittest.TextTestRunner(stream=io.StringIO(), verbosity=0).run(_suite)
assert _result.wasSuccessful(), "CPython Test_all.test_all_project_files did not pass"
print("Test_all::test_all_project_files: ok")
