# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "urllib"
# dimension = "behavior"
# case = "pathname_tests__test_quoting"
# subject = "cpython.test_urllib.Pathname_Tests.test_quoting"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_urllib.py"
# status = "filled"
# ///
# Auto-ported from CPython 3.12 test_urllib.py::Pathname_Tests::test_quoting
"""Auto-ported test: Pathname_Tests::test_quoting (CPython 3.12 oracle)."""


import urllib.parse
import urllib.request
import urllib.error
import http.client
import email.message
import io
import unittest
from unittest.mock import patch
from test import support
from test.support import os_helper
from test.support import socket_helper
from test.support import warnings_helper
import os
import sys
import tempfile
from base64 import b64encode
import collections


'Regression tests for what was in Python 2\'s "urllib" module'

try:
    import ssl
except ImportError:
    ssl = None

if not socket_helper.has_gethostname:
    raise unittest.SkipTest('test requires gethostname()')

def hexescape(char):
    """Escape char as RFC 2396 specifies"""
    hex_repr = hex(ord(char))[2:].upper()
    if len(hex_repr) == 1:
        hex_repr = '0%s' % hex_repr
    return '%' + hex_repr

_urlopener = None

def urlopen(url, data=None, proxies=None):
    """urlopen(url [, data]) -> open file-like object"""
    global _urlopener
    if proxies is not None:
        opener = urllib.request.FancyURLopener(proxies=proxies)
    elif not _urlopener:
        opener = FancyURLopener()
        _urlopener = opener
    else:
        opener = _urlopener
    if data is None:
        return opener.open(url)
    else:
        return opener.open(url, data)

def FancyURLopener():
    with warnings_helper.check_warnings(('FancyURLopener style of invoking requests is deprecated.', DeprecationWarning)):
        return urllib.request.FancyURLopener()

def fakehttp(fakedata, mock_close=False):

    class FakeSocket(io.BytesIO):
        io_refs = 1

        def sendall(self, data):
            FakeHTTPConnection.buf = data

        def makefile(self, *args, **kwds):
            self.io_refs += 1
            return self

        def read(self, amt=None):
            if self.closed:
                return b''
            return io.BytesIO.read(self, amt)

        def readline(self, length=None):
            if self.closed:
                return b''
            return io.BytesIO.readline(self, length)

        def close(self):
            self.io_refs -= 1
            if self.io_refs == 0:
                io.BytesIO.close(self)

    class FakeHTTPConnection(http.client.HTTPConnection):
        buf = None

        def connect(self):
            self.sock = FakeSocket(self.fakedata)
            type(self).fakesock = self.sock
        if mock_close:

            def close(self):
                pass
    FakeHTTPConnection.fakedata = fakedata
    return FakeHTTPConnection

class FakeHTTPMixin(object):

    def fakehttp(self, fakedata, mock_close=False):
        fake_http_class = fakehttp(fakedata, mock_close=mock_close)
        self._connection_class = http.client.HTTPConnection
        http.client.HTTPConnection = fake_http_class

    def unfakehttp(self):
        http.client.HTTPConnection = self._connection_class

class FakeFTPMixin(object):

    def fakeftp(self):

        class FakeFtpWrapper(object):

            def __init__(self, user, passwd, host, port, dirs, timeout=None, persistent=True):
                pass

            def retrfile(self, file, type):
                return (io.BytesIO(), 0)

            def close(self):
                pass
        self._ftpwrapper_class = urllib.request.ftpwrapper
        urllib.request.ftpwrapper = FakeFtpWrapper

    def unfakeftp(self):
        urllib.request.ftpwrapper = self._ftpwrapper_class


# --- test body ---
given = os.path.join('needs', 'quot=ing', 'here')
expect = 'needs/%s/here' % urllib.parse.quote('quot=ing')
result = urllib.request.pathname2url(given)

assert expect == result
expect = given
result = urllib.request.url2pathname(result)

assert expect == result
given = os.path.join('make sure', 'using_quote')
expect = '%s/using_quote' % urllib.parse.quote('make sure')
result = urllib.request.pathname2url(given)

assert expect == result
given = 'make+sure/using_unquote'
expect = os.path.join('make+sure', 'using_unquote')
result = urllib.request.url2pathname(given)

assert expect == result
print("Pathname_Tests::test_quoting: ok")
