# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "find"
# dimension = "behavior"
# case = "find_library_linux__test_find_library_with_gcc_ucae2689"
# subject = "cpython.test_find.FindLibraryLinux.test_find_library_with_gcc"
# kind = "semantic"
# xfail = "auto-extracted CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_ctypes/test_find.py"
# status = "filled"
# ///
# mamba-xfail: auto-extracted CPython test; mamba promotion pending
import unittest, io
from test.test_ctypes import test_find
_suite = unittest.defaultTestLoader.loadTestsFromName("FindLibraryLinux.test_find_library_with_gcc", test_find)
_result = unittest.TextTestRunner(stream=io.StringIO(), verbosity=0).run(_suite)
assert _result.wasSuccessful(), "CPython FindLibraryLinux.test_find_library_with_gcc did not pass"
print("FindLibraryLinux::test_find_library_with_gcc: ok")
