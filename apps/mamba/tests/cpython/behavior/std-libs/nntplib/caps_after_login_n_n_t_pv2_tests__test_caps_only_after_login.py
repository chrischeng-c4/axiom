# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "nntplib"
# dimension = "behavior"
# case = "caps_after_login_n_n_t_pv2_tests__test_caps_only_after_login"
# subject = "cpython.test_nntplib.CapsAfterLoginNNTPv2Tests.test_caps_only_after_login"
# kind = "semantic"
# xfail = "auto-extracted CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_nntplib.py"
# status = "filled"
# ///
# mamba-xfail: auto-extracted CPython test; mamba promotion pending
import unittest, io
from test import test_nntplib
_suite = unittest.defaultTestLoader.loadTestsFromName("CapsAfterLoginNNTPv2Tests.test_caps_only_after_login", test_nntplib)
_result = unittest.TextTestRunner(stream=io.StringIO(), verbosity=0).run(_suite)
assert _result.wasSuccessful(), "CPython CapsAfterLoginNNTPv2Tests.test_caps_only_after_login did not pass"
print("CapsAfterLoginNNTPv2Tests::test_caps_only_after_login: ok")
