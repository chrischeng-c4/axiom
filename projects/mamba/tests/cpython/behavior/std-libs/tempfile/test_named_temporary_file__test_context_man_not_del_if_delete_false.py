# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "tempfile"
# dimension = "behavior"
# case = "test_named_temporary_file__test_context_man_not_del_if_delete_false"
# subject = "cpython.test_tempfile.TestNamedTemporaryFile.test_context_man_not_del_if_delete_false"
# kind = "semantic"
# xfail = "auto-extracted CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_tempfile.py"
# status = "filled"
# ///
# mamba-xfail: auto-extracted CPython test; mamba promotion pending
import unittest, io
from test import test_tempfile
_suite = unittest.defaultTestLoader.loadTestsFromName("TestNamedTemporaryFile.test_context_man_not_del_if_delete_false", test_tempfile)
_result = unittest.TextTestRunner(stream=io.StringIO(), verbosity=0).run(_suite)
assert _result.wasSuccessful(), "CPython TestNamedTemporaryFile.test_context_man_not_del_if_delete_false did not pass"
print("TestNamedTemporaryFile::test_context_man_not_del_if_delete_false: ok")
