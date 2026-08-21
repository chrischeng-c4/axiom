# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "sslproto"
# dimension = "behavior"
# case = "ssl_proto_handshake_tests__test_close_during_ssl_over_ssl_uc9a7ee8"
# subject = "cpython.test_sslproto.SslProtoHandshakeTests.test_close_during_ssl_over_ssl"
# kind = "semantic"
# xfail = "auto-extracted CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_asyncio/test_sslproto.py"
# status = "filled"
# ///
# mamba-xfail: auto-extracted CPython test; mamba promotion pending
import unittest, io
from test.test_asyncio import test_sslproto
_suite = unittest.defaultTestLoader.loadTestsFromName("SslProtoHandshakeTests.test_close_during_ssl_over_ssl", test_sslproto)
_result = unittest.TextTestRunner(stream=io.StringIO(), verbosity=0).run(_suite)
assert _result.wasSuccessful(), "CPython SslProtoHandshakeTests.test_close_during_ssl_over_ssl did not pass"
print("SslProtoHandshakeTests::test_close_during_ssl_over_ssl: ok")
