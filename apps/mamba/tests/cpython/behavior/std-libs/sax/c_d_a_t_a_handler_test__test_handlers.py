# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "sax"
# dimension = "behavior"
# case = "c_d_a_t_a_handler_test__test_handlers"
# subject = "cpython.test_sax.CDATAHandlerTest.test_handlers"
# kind = "semantic"
# xfail = "auto-extracted CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_sax.py"
# status = "filled"
# ///
# mamba-xfail: auto-extracted CPython test; mamba promotion pending
import unittest, io
from test import test_sax
_suite = unittest.defaultTestLoader.loadTestsFromName("CDATAHandlerTest.test_handlers", test_sax)
_result = unittest.TextTestRunner(stream=io.StringIO(), verbosity=0).run(_suite)
assert _result.wasSuccessful(), "CPython CDATAHandlerTest.test_handlers did not pass"
print("CDATAHandlerTest::test_handlers: ok")
