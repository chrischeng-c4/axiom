# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "asyncio"
# dimension = "behavior"
# case = "selector_start_tls_tests__test_buf_feed_data"
# subject = "cpython.test_sslproto.SelectorStartTLSTests.test_buf_feed_data"
# kind = "semantic"
# xfail = "auto-ported CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_asyncio/test_sslproto.py"
# status = "filled"
# ///
# mamba-xfail: auto-ported CPython test; mamba promotion pending
# Auto-ported from CPython 3.12 test_sslproto.py::SelectorStartTLSTests::test_buf_feed_data
"""Auto-ported test: SelectorStartTLSTests::test_buf_feed_data (CPython 3.12 oracle)."""


import logging
import socket
import unittest
import weakref
from test import support
from test.support import socket_helper
from unittest import mock
import asyncio
from asyncio import log
from asyncio import protocols
from asyncio import sslproto
from test.test_asyncio import utils as test_utils
from test.test_asyncio import functional as func_tests


'Tests for asyncio/sslproto.py.'

try:
    import ssl
except ImportError:
    ssl = None

def tearDownModule():
    asyncio.set_event_loop_policy(None)


# --- test body ---
PAYLOAD_SIZE = 1024 * 100
TIMEOUT = support.LONG_TIMEOUT

def new_loop():
    return asyncio.SelectorEventLoop()

class Proto(asyncio.BufferedProtocol):

    def __init__(self, bufsize, usemv):
        self.buf = bytearray(bufsize)
        self.mv = memoryview(self.buf)
        self.data = b''
        self.usemv = usemv

    def get_buffer(self, sizehint):
        if self.usemv:
            return self.mv
        else:
            return self.buf

    def buffer_updated(self, nsize):
        if self.usemv:
            self.data += self.mv[:nsize]
        else:
            self.data += self.buf[:nsize]
for usemv in [False, True]:
    proto = Proto(1, usemv)
    protocols._feed_data_to_buffered_proto(proto, b'12345')

    assert proto.data == b'12345'
    proto = Proto(2, usemv)
    protocols._feed_data_to_buffered_proto(proto, b'12345')

    assert proto.data == b'12345'
    proto = Proto(2, usemv)
    protocols._feed_data_to_buffered_proto(proto, b'1234')

    assert proto.data == b'1234'
    proto = Proto(4, usemv)
    protocols._feed_data_to_buffered_proto(proto, b'1234')

    assert proto.data == b'1234'
    proto = Proto(100, usemv)
    protocols._feed_data_to_buffered_proto(proto, b'12345')

    assert proto.data == b'12345'
    proto = Proto(0, usemv)
    try:
        protocols._feed_data_to_buffered_proto(proto, b'12345')
        raise AssertionError('expected RuntimeError')
    except RuntimeError as _aR_e:
        import re as _re_aR
        assert _re_aR.search('empty buffer', str(_aR_e))
print("SelectorStartTLSTests::test_buf_feed_data: ok")
