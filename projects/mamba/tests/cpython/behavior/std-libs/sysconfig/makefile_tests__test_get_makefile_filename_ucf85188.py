# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "sysconfig"
# dimension = "behavior"
# case = "makefile_tests__test_get_makefile_filename_ucf85188"
# subject = "cpython.test_sysconfig.MakefileTests.test_get_makefile_filename"
# kind = "semantic"
# xfail = "auto-extracted CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_sysconfig.py"
# status = "filled"
# ///
# mamba-xfail: auto-extracted CPython test; mamba promotion pending
import unittest, io
from test import test_sysconfig
_suite = unittest.defaultTestLoader.loadTestsFromName("MakefileTests.test_get_makefile_filename", test_sysconfig)
_result = unittest.TextTestRunner(stream=io.StringIO(), verbosity=0).run(_suite)
assert _result.wasSuccessful(), "CPython MakefileTests.test_get_makefile_filename did not pass"
print("MakefileTests::test_get_makefile_filename: ok")
