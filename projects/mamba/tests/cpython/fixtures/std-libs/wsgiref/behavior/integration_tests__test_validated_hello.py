# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "wsgiref"
# dimension = "behavior"
# case = "integration_tests__test_validated_hello"
# subject = "cpython.test_wsgiref.IntegrationTests.test_validated_hello"
# kind = "semantic"
# xfail = "auto-ported CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_wsgiref.py"
# status = "filled"
# ///
# mamba-xfail: auto-ported CPython test; mamba promotion pending
# Auto-ported from CPython 3.12 test_wsgiref.py::IntegrationTests::test_validated_hello
"""Auto-ported test: IntegrationTests::test_validated_hello (CPython 3.12 oracle)."""


from unittest import mock
from test import support
from test.support import socket_helper
from test.test_httpservers import NoLogRequestHandler
from unittest import TestCase
from wsgiref.util import setup_testing_defaults
from wsgiref.headers import Headers
from wsgiref.handlers import BaseHandler, BaseCGIHandler, SimpleHandler
from wsgiref import util
from wsgiref.validate import validator
from wsgiref.simple_server import WSGIServer, WSGIRequestHandler
from wsgiref.simple_server import make_server
from http.client import HTTPConnection
from io import StringIO, BytesIO, BufferedReader
from socketserver import BaseServer
from platform import python_implementation
import os
import re
import signal
import sys
import threading
import unittest


class MockServer(WSGIServer):
    """Non-socket HTTP server"""

    def __init__(self, server_address, RequestHandlerClass):
        BaseServer.__init__(self, server_address, RequestHandlerClass)
        self.server_bind()

    def server_bind(self):
        host, port = self.server_address
        self.server_name = host
        self.server_port = port
        self.setup_environ()

class MockHandler(WSGIRequestHandler):
    """Non-socket HTTP handler"""

    def setup(self):
        self.connection = self.request
        self.rfile, self.wfile = self.connection

    def finish(self):
        pass

def hello_app(environ, start_response):
    start_response('200 OK', [('Content-Type', 'text/plain'), ('Date', 'Mon, 05 Jun 2006 18:49:54 GMT')])
    return [b'Hello, world!']

def header_app(environ, start_response):
    start_response('200 OK', [('Content-Type', 'text/plain'), ('Date', 'Mon, 05 Jun 2006 18:49:54 GMT')])
    return [';'.join([environ['HTTP_X_TEST_HEADER'], environ['QUERY_STRING'], environ['PATH_INFO']]).encode('iso-8859-1')]

def run_amock(app=hello_app, data=b'GET / HTTP/1.0\n\n'):
    server = make_server('', 80, app, MockServer, MockHandler)
    inp = BufferedReader(BytesIO(data))
    out = BytesIO()
    olderr = sys.stderr
    err = sys.stderr = StringIO()
    try:
        server.finish_request((inp, out), ('127.0.0.1', 8888))
    finally:
        sys.stderr = olderr
    return (out.getvalue(), err.getvalue())

def compare_generic_iter(make_it, match):
    """Utility to compare a generic iterator with an iterable

    This tests the iterator using iter()/next().
    'make_it' must be a function returning a fresh
    iterator to be tested (since this may test the iterator twice)."""
    it = make_it()
    if not iter(it) is it:
        raise AssertionError
    for item in match:
        if not next(it) == item:
            raise AssertionError
    try:
        next(it)
    except StopIteration:
        pass
    else:
        raise AssertionError('Too many items from .__next__()', it)

class ErrorHandler(BaseCGIHandler):
    """Simple handler subclass for testing BaseHandler"""
    os_environ = dict(os.environ.items())

    def __init__(self, **kw):
        setup_testing_defaults(kw)
        BaseCGIHandler.__init__(self, BytesIO(), BytesIO(), StringIO(), kw, multithread=True, multiprocess=True)


# --- test body ---
def check_hello(out, has_length=True):
    pyver = python_implementation() + '/' + sys.version.split()[0]

    assert out == ('HTTP/1.0 200 OK\r\nServer: WSGIServer/0.2 ' + pyver + '\r\nContent-Type: text/plain\r\nDate: Mon, 05 Jun 2006 18:49:54 GMT\r\n' + (has_length and 'Content-Length: 13\r\n' or '') + '\r\nHello, world!').encode('iso-8859-1')
out, err = run_amock(validator(hello_app))
check_hello(out, has_length=False)
print("IntegrationTests::test_validated_hello: ok")
