# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "makefile"
# dimension = "behavior"
# case = "test_makefile__test_makefile_test_folders_ucd0530d"
# subject = "cpython.test_makefile.TestMakefile.test_makefile_test_folders"
# kind = "semantic"
# xfail = "auto-extracted CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_tools/test_makefile.py"
# status = "filled"
# ///
# mamba-xfail: auto-extracted CPython test; mamba promotion pending
import unittest, io
from test.test_tools import test_makefile
_suite = unittest.defaultTestLoader.loadTestsFromName("TestMakefile.test_makefile_test_folders", test_makefile)
_result = unittest.TextTestRunner(stream=io.StringIO(), verbosity=0).run(_suite)
assert _result.wasSuccessful(), "CPython TestMakefile.test_makefile_test_folders did not pass"
print("TestMakefile::test_makefile_test_folders: ok")
