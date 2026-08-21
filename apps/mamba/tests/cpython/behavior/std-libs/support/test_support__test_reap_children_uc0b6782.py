# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "support"
# dimension = "behavior"
# case = "test_support__test_reap_children_uc0b6782"
# subject = "cpython.test_support.TestSupport.test_reap_children"
# kind = "semantic"
# xfail = "auto-extracted CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_support.py"
# status = "filled"
# ///
# mamba-xfail: auto-extracted CPython test; mamba promotion pending
import unittest, io
from test import test_support
_suite = unittest.defaultTestLoader.loadTestsFromName("TestSupport.test_reap_children", test_support)
_result = unittest.TextTestRunner(stream=io.StringIO(), verbosity=0).run(_suite)
assert _result.wasSuccessful(), "CPython TestSupport.test_reap_children did not pass"
print("TestSupport::test_reap_children: ok")
