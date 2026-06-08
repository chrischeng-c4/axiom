# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "support"
# dimension = "behavior"
# case = "test_support__test_bind_port_ucd704ca"
# subject = "cpython.test_support.TestSupport.test_bind_port"
# kind = "semantic"
# xfail = "auto-extracted CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_support.py"
# status = "filled"
# ///
# mamba-xfail: auto-extracted CPython test; mamba promotion pending
import unittest, io
from test import test_support
_suite = unittest.defaultTestLoader.loadTestsFromName("TestSupport.test_bind_port", test_support)
_result = unittest.TextTestRunner(stream=io.StringIO(), verbosity=0).run(_suite)
assert _result.wasSuccessful(), "CPython TestSupport.test_bind_port did not pass"
print("TestSupport::test_bind_port: ok")
