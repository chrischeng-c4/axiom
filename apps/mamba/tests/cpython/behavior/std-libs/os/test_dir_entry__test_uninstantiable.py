# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "os"
# dimension = "behavior"
# case = "test_dir_entry__test_uninstantiable"
# subject = "cpython.test_os.TestDirEntry.test_uninstantiable"
# kind = "semantic"
# xfail = "auto-extracted CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_os.py"
# status = "filled"
# ///
# mamba-xfail: auto-extracted CPython test; mamba promotion pending
import unittest, io
from test import test_os
_suite = unittest.defaultTestLoader.loadTestsFromName("TestDirEntry.test_uninstantiable", test_os)
_result = unittest.TextTestRunner(stream=io.StringIO(), verbosity=0).run(_suite)
assert _result.wasSuccessful(), "CPython TestDirEntry.test_uninstantiable did not pass"
print("TestDirEntry::test_uninstantiable: ok")
