# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "optparse"
# dimension = "behavior"
# case = "test_extend_add_types__test_filetype_notfile"
# subject = "cpython.test_optparse.TestExtendAddTypes.test_filetype_notfile"
# kind = "semantic"
# xfail = "auto-extracted CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_optparse.py"
# status = "filled"
# ///
# mamba-xfail: auto-extracted CPython test; mamba promotion pending
import unittest, io
from test import test_optparse
_suite = unittest.defaultTestLoader.loadTestsFromName("TestExtendAddTypes.test_filetype_notfile", test_optparse)
_result = unittest.TextTestRunner(stream=io.StringIO(), verbosity=0).run(_suite)
assert _result.wasSuccessful(), "CPython TestExtendAddTypes.test_filetype_notfile did not pass"
print("TestExtendAddTypes::test_filetype_notfile: ok")
