# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "tempfile"
# dimension = "behavior"
# case = "test_get_temp_prefix__test_sane_template"
# subject = "cpython.test_tempfile.TestGetTempPrefix.test_sane_template"
# kind = "semantic"
# xfail = "auto-extracted CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_tempfile.py"
# status = "filled"
# ///
# mamba-xfail: auto-extracted CPython test; mamba promotion pending
import unittest, io
from test import test_tempfile
_suite = unittest.defaultTestLoader.loadTestsFromName("TestGetTempPrefix.test_sane_template", test_tempfile)
_result = unittest.TextTestRunner(stream=io.StringIO(), verbosity=0).run(_suite)
assert _result.wasSuccessful(), "CPython TestGetTempPrefix.test_sane_template did not pass"
print("TestGetTempPrefix::test_sane_template: ok")
