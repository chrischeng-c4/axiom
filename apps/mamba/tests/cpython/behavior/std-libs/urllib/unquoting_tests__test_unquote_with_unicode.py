# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "urllib"
# dimension = "behavior"
# case = "unquoting_tests__test_unquote_with_unicode"
# subject = "cpython.test_urllib.UnquotingTests.test_unquote_with_unicode"
# kind = "semantic"
# xfail = "auto-ported CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_urllib.py"
# status = "filled"
# ///
# mamba-xfail: auto-ported CPython test; mamba promotion pending
# Auto-ported from CPython 3.12 test_urllib.py::UnquotingTests::test_unquote_with_unicode
"""Auto-ported test: UnquotingTests::test_unquote_with_unicode (CPython 3.12 oracle)."""


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
given = 'br%C3%BCckner_sapporo_20050930.doc'
expect = 'brückner_sapporo_20050930.doc'
result = urllib.parse.unquote(given)

assert expect == result
result = urllib.parse.unquote(given, encoding=None, errors=None)

assert expect == result
result = urllib.parse.unquote('br%FCckner_sapporo_20050930.doc', encoding='latin-1')
expect = 'brückner_sapporo_20050930.doc'

assert expect == result
given = '%E6%BC%A2%E5%AD%97'
expect = '漢字'
result = urllib.parse.unquote(given)

assert expect == result
given = '%F3%B1'
expect = '�'
result = urllib.parse.unquote(given)

assert expect == result
result = urllib.parse.unquote(given, errors='replace')

assert expect == result
given = '%F3%B1'
expect = ''
result = urllib.parse.unquote(given, errors='ignore')

assert expect == result
result = urllib.parse.unquote('漢%C3%BC')
expect = '漢ü'

assert expect == result
result = urllib.parse.unquote('漢%FC', encoding='latin-1')
expect = '漢ü'

assert expect == result
print("UnquotingTests::test_unquote_with_unicode: ok")
