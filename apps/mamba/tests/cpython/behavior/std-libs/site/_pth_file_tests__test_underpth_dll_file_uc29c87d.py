# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "site"
# dimension = "behavior"
# case = "_pth_file_tests__test_underpth_dll_file_uc29c87d"
# subject = "cpython.test_site._pthFileTests.test_underpth_dll_file"
# kind = "semantic"
# xfail = "auto-extracted CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_site.py"
# status = "filled"
# ///
# mamba-xfail: auto-extracted CPython test; mamba promotion pending
import unittest, io
from test import test_site
_suite = unittest.defaultTestLoader.loadTestsFromName("_pthFileTests.test_underpth_dll_file", test_site)
_result = unittest.TextTestRunner(stream=io.StringIO(), verbosity=0).run(_suite)
assert _result.wasSuccessful(), "CPython _pthFileTests.test_underpth_dll_file did not pass"
print("_pthFileTests::test_underpth_dll_file: ok")
