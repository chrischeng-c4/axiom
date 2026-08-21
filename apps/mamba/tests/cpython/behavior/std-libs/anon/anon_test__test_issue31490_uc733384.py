# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "anon"
# dimension = "behavior"
# case = "anon_test__test_issue31490_uc733384"
# subject = "cpython.test_anon.AnonTest.test_issue31490"
# kind = "semantic"
# xfail = "auto-extracted CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_ctypes/test_anon.py"
# status = "filled"
# ///
# mamba-xfail: auto-extracted CPython test; mamba promotion pending
import unittest, io
from test.test_ctypes import test_anon
_suite = unittest.defaultTestLoader.loadTestsFromName("AnonTest.test_issue31490", test_anon)
_result = unittest.TextTestRunner(stream=io.StringIO(), verbosity=0).run(_suite)
assert _result.wasSuccessful(), "CPython AnonTest.test_issue31490 did not pass"
print("AnonTest::test_issue31490: ok")
