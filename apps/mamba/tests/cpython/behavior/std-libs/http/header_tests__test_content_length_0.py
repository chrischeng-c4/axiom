# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "http"
# dimension = "behavior"
# case = "header_tests__test_content_length_0"
# subject = "cpython.test_httplib.HeaderTests.test_content_length_0"
# kind = "semantic"
# xfail = "auto-ported CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_httplib.py"
# status = "filled"
# ///
# mamba-xfail: auto-ported CPython test; mamba promotion pending
# Auto-ported from CPython 3.12 test_httplib.py::HeaderTests::test_content_length_0
"""Auto-ported test: HeaderTests::test_content_length_0 (CPython 3.12 oracle)."""


import enum
import errno
from http import client, HTTPStatus
import io
import itertools
import os
import array
import re
import socket
import threading
import unittest
from unittest import mock
from test import support
from test.support import os_helper
from test.support import socket_helper


TestCase = unittest.TestCase

support.requires_working_socket(module=True)

here = os.path.dirname(__file__)

CERT_localhost = os.path.join(here, 'certdata', 'keycert.pem')

CERT_fakehostname = os.path.join(here, 'certdata', 'keycert2.pem')

CERT_selfsigned_pythontestdotnet = os.path.join(here, 'certdata', 'selfsigned_pythontestdotnet.pem')

chunked_start = 'HTTP/1.1 200 OK\r\nTransfer-Encoding: chunked\r\n\r\na\r\nhello worl\r\n3\r\nd! \r\n8\r\nand now \r\n22\r\nfor something completely different\r\n'

chunked_expected = b'hello world! and now for something completely different'

chunk_extension = ';foo=bar'

last_chunk = '0\r\n'

last_chunk_extended = '0' + chunk_extension + '\r\n'

trailers = 'X-Dummy: foo\r\nX-Dumm2: bar\r\n'

chunked_end = '\r\n'

HOST = socket_helper.HOST

class FakeSocket:

    def __init__(self, text, fileclass=io.BytesIO, host=None, port=None):
        if isinstance(text, str):
            text = text.encode('ascii')
        self.text = text
        self.fileclass = fileclass
        self.data = b''
        self.sendall_calls = 0
        self.file_closed = False
        self.host = host
        self.port = port

    def sendall(self, data):
        self.sendall_calls += 1
        self.data += data

    def makefile(self, mode, bufsize=None):
        if mode != 'r' and mode != 'rb':
            raise client.UnimplementedFileMode()
        self.file = self.fileclass(self.text)
        self.file.close = self.file_close
        return self.file

    def file_close(self):
        self.file_closed = True

    def close(self):
        pass

    def setsockopt(self, level, optname, value):
        pass

class EPipeSocket(FakeSocket):

    def __init__(self, text, pipe_trigger):
        FakeSocket.__init__(self, text)
        self.pipe_trigger = pipe_trigger

    def sendall(self, data):
        if self.pipe_trigger in data:
            raise OSError(errno.EPIPE, 'gotcha')
        self.data += data

    def close(self):
        pass

class NoEOFBytesIO(io.BytesIO):
    """Like BytesIO, but raises AssertionError on EOF.

    This is used below to test that http.client doesn't try to read
    more from the underlying file than it should.
    """

    def read(self, n=-1):
        data = io.BytesIO.read(self, n)
        if data == b'':
            raise AssertionError('caller tried to read past EOF')
        return data

    def readline(self, length=None):
        data = io.BytesIO.readline(self, length)
        if data == b'':
            raise AssertionError('caller tried to read past EOF')
        return data

class FakeSocketHTTPConnection(client.HTTPConnection):
    """HTTPConnection subclass using FakeSocket; counts connect() calls"""

    def __init__(self, *args):
        self.connections = 0
        super().__init__('example.com')
        self.fake_socket_args = args
        self._create_connection = self.create_connection

    def connect(self):
        """Count the number of times connect() is invoked"""
        self.connections += 1
        return super().connect()

    def create_connection(self, *pos, **kw):
        return FakeSocket(*self.fake_socket_args)

class Readliner:
    """
    a simple readline class that uses an arbitrary read function and buffering
    """

    def __init__(self, readfunc):
        self.readfunc = readfunc
        self.remainder = b''

    def readline(self, limit):
        data = []
        datalen = 0
        read = self.remainder
        try:
            while True:
                idx = read.find(b'\n')
                if idx != -1:
                    break
                if datalen + len(read) >= limit:
                    idx = limit - datalen - 1
                data.append(read)
                read = self.readfunc()
                if not read:
                    idx = 0
                    break
            idx += 1
            data.append(read[:idx])
            self.remainder = read[idx:]
            return b''.join(data)
        except:
            self.remainder = b''.join(data)
            raise


# --- test body ---
class ContentLengthChecker(list):

    def __init__(self):
        list.__init__(self)
        self.content_length = None

    def append(self, item):
        kv = item.split(b':', 1)
        if len(kv) > 1 and kv[0].lower() == b'content-length':
            self.content_length = kv[1].strip()
        list.append(self, item)
bodies = (None, '')
methods_with_body = ('PUT', 'POST', 'PATCH')
for method, body in itertools.product(methods_with_body, bodies):
    conn = client.HTTPConnection('example.com')
    conn.sock = FakeSocket(None)
    conn._buffer = ContentLengthChecker()
    conn.request(method, '/', body)

    assert conn._buffer.content_length == b'0'
methods_without_body = ('GET', 'CONNECT', 'DELETE', 'HEAD', 'OPTIONS', 'TRACE')
for method in methods_without_body:
    conn = client.HTTPConnection('example.com')
    conn.sock = FakeSocket(None)
    conn._buffer = ContentLengthChecker()
    conn.request(method, '/', None)

    assert conn._buffer.content_length == None
for method in methods_without_body:
    conn = client.HTTPConnection('example.com')
    conn.sock = FakeSocket(None)
    conn._buffer = ContentLengthChecker()
    conn.request(method, '/', '')

    assert conn._buffer.content_length == b'0'
for method in itertools.chain(methods_without_body, methods_with_body):
    conn = client.HTTPConnection('example.com')
    conn.sock = FakeSocket(None)
    conn._buffer = ContentLengthChecker()
    conn.request(method, '/', ' ')

    assert conn._buffer.content_length == b'1'
print("HeaderTests::test_content_length_0: ok")
