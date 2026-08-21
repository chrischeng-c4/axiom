# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "shutil"
# dimension = "behavior"
# case = "test_archives__test_make_archive_cwd_supports_root_dir"
# subject = "cpython.test_shutil.TestArchives.test_make_archive_cwd_supports_root_dir"
# kind = "semantic"
# xfail = "auto-extracted CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_shutil.py"
# status = "filled"
# ///
# mamba-xfail: auto-extracted CPython test; mamba promotion pending
import unittest, io
from test import test_shutil
_suite = unittest.defaultTestLoader.loadTestsFromName("TestArchives.test_make_archive_cwd_supports_root_dir", test_shutil)
_result = unittest.TextTestRunner(stream=io.StringIO(), verbosity=0).run(_suite)
assert _result.wasSuccessful(), "CPython TestArchives.test_make_archive_cwd_supports_root_dir did not pass"
print("TestArchives::test_make_archive_cwd_supports_root_dir: ok")
