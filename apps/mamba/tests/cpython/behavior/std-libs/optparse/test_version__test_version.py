# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "optparse"
# dimension = "behavior"
# case = "test_version__test_version"
# subject = "cpython.test_optparse.TestVersion.test_version"
# kind = "semantic"
# xfail = "auto-extracted CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_optparse.py"
# status = "filled"
# ///
# mamba-xfail: auto-extracted CPython test; mamba promotion pending
import unittest, io
from test import test_optparse
_suite = unittest.defaultTestLoader.loadTestsFromName("TestVersion.test_version", test_optparse)
_result = unittest.TextTestRunner(stream=io.StringIO(), verbosity=0).run(_suite)
assert _result.wasSuccessful(), "CPython TestVersion.test_version did not pass"
print("TestVersion::test_version: ok")
