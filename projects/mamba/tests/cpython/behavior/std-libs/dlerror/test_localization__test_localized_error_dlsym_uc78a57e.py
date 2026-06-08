# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "dlerror"
# dimension = "behavior"
# case = "test_localization__test_localized_error_dlsym_uc78a57e"
# subject = "cpython.test_dlerror.TestLocalization.test_localized_error_dlsym"
# kind = "semantic"
# xfail = "auto-extracted CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_ctypes/test_dlerror.py"
# status = "filled"
# ///
# mamba-xfail: auto-extracted CPython test; mamba promotion pending
import unittest, io
from test.test_ctypes import test_dlerror
_suite = unittest.defaultTestLoader.loadTestsFromName("TestLocalization.test_localized_error_dlsym", test_dlerror)
_result = unittest.TextTestRunner(stream=io.StringIO(), verbosity=0).run(_suite)
assert _result.wasSuccessful(), "CPython TestLocalization.test_localized_error_dlsym did not pass"
print("TestLocalization::test_localized_error_dlsym: ok")
