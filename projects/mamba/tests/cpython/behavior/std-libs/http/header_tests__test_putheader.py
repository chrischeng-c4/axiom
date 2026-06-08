# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "http"
# dimension = "behavior"
# case = "header_tests__test_putheader"
# subject = "cpython.test_httplib.HeaderTests.test_putheader"
# kind = "semantic"
# xfail = "auto-ported CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_httplib.py"
# status = "filled"
# ///
# mamba-xfail: auto-ported CPython test; mamba promotion pending
# Auto-ported from CPython 3.12 test_httplib.py::HeaderTests::test_putheader
"""Auto-ported test: HeaderTests::test_putheader (CPython 3.12 oracle)."""


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
conn = client.HTTPConnection('example.com')
conn.sock = FakeSocket(None)
conn.putrequest('GET', '/')
conn.putheader('Content-length', 42)

assert b'Content-length: 42' in conn._buffer
conn.putheader('Foo', ' bar ')

assert b'Foo:  bar ' in conn._buffer
conn.putheader('Bar', '\tbaz\t')

assert b'Bar: \tbaz\t' in conn._buffer
conn.putheader('Authorization', 'Bearer mytoken')

assert b'Authorization: Bearer mytoken' in conn._buffer
conn.putheader('IterHeader', 'IterA', 'IterB')

assert b'IterHeader: IterA\r\n\tIterB' in conn._buffer
conn.putheader('LatinHeader', b'\xff')

assert b'LatinHeader: \xff' in conn._buffer
conn.putheader('Utf8Header', b'\xc3\x80')

assert b'Utf8Header: \xc3\x80' in conn._buffer
conn.putheader('C1-Control', b'next\x85line')

assert b'C1-Control: next\x85line' in conn._buffer
conn.putheader('Embedded-Fold-Space', 'is\r\n allowed')

assert b'Embedded-Fold-Space: is\r\n allowed' in conn._buffer
conn.putheader('Embedded-Fold-Tab', 'is\r\n\tallowed')

assert b'Embedded-Fold-Tab: is\r\n\tallowed' in conn._buffer
conn.putheader('Key Space', 'value')

assert b'Key Space: value' in conn._buffer
conn.putheader('KeySpace ', 'value')

assert b'KeySpace : value' in conn._buffer
conn.putheader(b'Nonbreak\xa0Space', 'value')

assert b'Nonbreak\xa0Space: value' in conn._buffer
conn.putheader(b'\xa0NonbreakSpace', 'value')

assert b'\xa0NonbreakSpace: value' in conn._buffer
print("HeaderTests::test_putheader: ok")
