# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "sslproto"
# dimension = "behavior"
# case = "ssl_proto_handshake_tests__test_write_after_closing_ucb5ee9b"
# subject = "cpython.test_sslproto.SslProtoHandshakeTests.test_write_after_closing"
# kind = "semantic"
# xfail = "auto-extracted CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_asyncio/test_sslproto.py"
# status = "filled"
# ///
# mamba-xfail: auto-extracted CPython test; mamba promotion pending
import unittest, io
from test.test_asyncio import test_sslproto
_suite = unittest.defaultTestLoader.loadTestsFromName("SslProtoHandshakeTests.test_write_after_closing", test_sslproto)
_result = unittest.TextTestRunner(stream=io.StringIO(), verbosity=0).run(_suite)
assert _result.wasSuccessful(), "CPython SslProtoHandshakeTests.test_write_after_closing did not pass"
print("SslProtoHandshakeTests::test_write_after_closing: ok")
