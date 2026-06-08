# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "dlerror"
# dimension = "behavior"
# case = "test_null_dlsym__test_null_dlsym_ucfb24d1"
# subject = "cpython.test_dlerror.TestNullDlsym.test_null_dlsym"
# kind = "semantic"
# xfail = "auto-extracted CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_ctypes/test_dlerror.py"
# status = "filled"
# ///
# mamba-xfail: auto-extracted CPython test; mamba promotion pending
import unittest, io
from test.test_ctypes import test_dlerror
_suite = unittest.defaultTestLoader.loadTestsFromName("TestNullDlsym.test_null_dlsym", test_dlerror)
_result = unittest.TextTestRunner(stream=io.StringIO(), verbosity=0).run(_suite)
assert _result.wasSuccessful(), "CPython TestNullDlsym.test_null_dlsym did not pass"
print("TestNullDlsym::test_null_dlsym: ok")
