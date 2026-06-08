# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "typing"
# dimension = "behavior"
# case = "protocol_tests__test_everything_implements_empty_protocol"
# subject = "cpython.test_typing.ProtocolTests.test_everything_implements_empty_protocol"
# kind = "semantic"
# xfail = "auto-extracted CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_typing.py"
# status = "filled"
# ///
# mamba-xfail: auto-extracted CPython test; mamba promotion pending
import unittest, io
from test import test_typing
_suite = unittest.defaultTestLoader.loadTestsFromName("ProtocolTests.test_everything_implements_empty_protocol", test_typing)
_result = unittest.TextTestRunner(stream=io.StringIO(), verbosity=0).run(_suite)
assert _result.wasSuccessful(), "CPython ProtocolTests.test_everything_implements_empty_protocol did not pass"
print("ProtocolTests::test_everything_implements_empty_protocol: ok")
