# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "shutil"
# dimension = "behavior"
# case = "test_archives__test_zipfile_vs_zip"
# subject = "cpython.test_shutil.TestArchives.test_zipfile_vs_zip"
# kind = "semantic"
# xfail = "auto-extracted CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_shutil.py"
# status = "filled"
# ///
# mamba-xfail: auto-extracted CPython test; mamba promotion pending
import unittest, io
from test import test_shutil
_suite = unittest.defaultTestLoader.loadTestsFromName("TestArchives.test_zipfile_vs_zip", test_shutil)
_result = unittest.TextTestRunner(stream=io.StringIO(), verbosity=0).run(_suite)
assert _result.wasSuccessful(), "CPython TestArchives.test_zipfile_vs_zip did not pass"
print("TestArchives::test_zipfile_vs_zip: ok")
