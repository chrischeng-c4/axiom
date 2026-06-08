# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "weakref"
# dimension = "behavior"
# case = "references_test_case__test_proxy_ref_uc25647f"
# subject = "cpython.test_weakref.ReferencesTestCase.test_proxy_ref"
# kind = "semantic"
# xfail = "auto-extracted CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_weakref.py"
# status = "filled"
# ///
# mamba-xfail: auto-extracted CPython test; mamba promotion pending
import unittest, io
from test import test_weakref
_suite = unittest.defaultTestLoader.loadTestsFromName("ReferencesTestCase.test_proxy_ref", test_weakref)
_result = unittest.TextTestRunner(stream=io.StringIO(), verbosity=0).run(_suite)
assert _result.wasSuccessful(), "CPython ReferencesTestCase.test_proxy_ref did not pass"
print("ReferencesTestCase::test_proxy_ref: ok")
