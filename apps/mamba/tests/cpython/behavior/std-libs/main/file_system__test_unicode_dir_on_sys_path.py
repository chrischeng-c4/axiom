# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "main"
# dimension = "behavior"
# case = "file_system__test_unicode_dir_on_sys_path"
# subject = "cpython.test_main.FileSystem.test_unicode_dir_on_sys_path"
# kind = "semantic"
# xfail = "auto-extracted CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_importlib/test_main.py"
# status = "filled"
# ///
# mamba-xfail: auto-extracted CPython test; mamba promotion pending
import unittest, io
from test.test_importlib import test_main
_suite = unittest.defaultTestLoader.loadTestsFromName("FileSystem.test_unicode_dir_on_sys_path", test_main)
_result = unittest.TextTestRunner(stream=io.StringIO(), verbosity=0).run(_suite)
assert _result.wasSuccessful(), "CPython FileSystem.test_unicode_dir_on_sys_path did not pass"
print("FileSystem::test_unicode_dir_on_sys_path: ok")
