# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "urllib"
# dimension = "behavior"
# case = "urlencode_tests__test_urlencode_encoding_doseq"
# subject = "cpython.test_urllib.urlencode_Tests.test_urlencode_encoding_doseq"
# kind = "semantic"
# xfail = "auto-ported CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_urllib.py"
# status = "filled"
# ///
# mamba-xfail: auto-ported CPython test; mamba promotion pending
# Auto-ported from CPython 3.12 test_urllib.py::urlencode_Tests::test_urlencode_encoding_doseq
"""Auto-ported test: urlencode_Tests::test_urlencode_encoding_doseq (CPython 3.12 oracle)."""


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
given = (('\xa0', 'Á'),)
expect = '%3F=%3F'
result = urllib.parse.urlencode(given, doseq=True, encoding='ASCII', errors='replace')

assert expect == result
given = (('\xa0', (1, 'Á')),)
expect = '%3F=1&%3F=%3F'
result = urllib.parse.urlencode(given, True, encoding='ASCII', errors='replace')

assert expect == result
given = (('\xa0', 'Á'),)
expect = '%C2%A0=%C3%81'
result = urllib.parse.urlencode(given, True)

assert expect == result
given = (('\xa0', (42, 'Á')),)
expect = '%C2%A0=42&%C2%A0=%C3%81'
result = urllib.parse.urlencode(given, True)

assert expect == result
given = (('\xa0', 'Á'),)
expect = '%A0=%C1'
result = urllib.parse.urlencode(given, True, encoding='latin-1')

assert expect == result
given = (('\xa0', (42, 'Á')),)
expect = '%A0=42&%A0=%C1'
result = urllib.parse.urlencode(given, True, encoding='latin-1')

assert expect == result
print("urlencode_Tests::test_urlencode_encoding_doseq: ok")
