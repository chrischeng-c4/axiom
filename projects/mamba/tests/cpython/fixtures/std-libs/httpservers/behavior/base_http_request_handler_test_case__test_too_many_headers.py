# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "httpservers"
# dimension = "behavior"
# case = "base_http_request_handler_test_case__test_too_many_headers"
# subject = "cpython.test_httpservers.BaseHTTPRequestHandlerTestCase.test_too_many_headers"
# kind = "semantic"
# xfail = "auto-ported CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_httpservers.py"
# status = "filled"
# ///
# mamba-xfail: auto-ported CPython test; mamba promotion pending
# Auto-ported from CPython 3.12 test_httpservers.py::BaseHTTPRequestHandlerTestCase::test_too_many_headers
"""Auto-ported test: BaseHTTPRequestHandlerTestCase::test_too_many_headers (CPython 3.12 oracle)."""


from collections import OrderedDict
from http.server import BaseHTTPRequestHandler, HTTPServer, SimpleHTTPRequestHandler, CGIHTTPRequestHandler
from http import server, HTTPStatus
import os
import socket
import sys
import re
import base64
import ntpath
import pathlib
import shutil
import email.message
import email.utils
import html
import http, http.client
import urllib.parse
import tempfile
import time
import datetime
import threading
from unittest import mock
import warnings
from io import BytesIO, StringIO
import unittest
from test import support
from test.support import os_helper
from test.support import threading_helper


'Unittests for the various HTTPServer modules.\n\nWritten by Cody A.W. Somerville <cody-somerville@ubuntu.com>,\nJosip Dzolonga, and Michael Otteneder for the 2007/08 GHOP contest.\n'

support.requires_working_socket(module=True)

class NoLogRequestHandler:

    def log_message(self, *args):
        pass

    def read(self, n=None):
        return ''

cgi_file1 = '#!%s\n\nprint("Content-type: text/html")\nprint()\nprint("Hello World")\n'

cgi_file2 = '#!%s\nimport os\nimport sys\nimport urllib.parse\n\nprint("Content-type: text/html")\nprint()\n\ncontent_length = int(os.environ["CONTENT_LENGTH"])\nquery_string = sys.stdin.buffer.read(content_length)\nparams = {key.decode("utf-8"): val.decode("utf-8")\n            for key, val in urllib.parse.parse_qsl(query_string)}\n\nprint("%%s, %%s, %%s" %% (params["spam"], params["eggs"], params["bacon"]))\n'

cgi_file4 = '#!%s\nimport os\n\nprint("Content-type: text/html")\nprint()\n\nprint(os.environ["%s"])\n'

cgi_file6 = '#!%s\nimport os\n\nprint("X-ambv: was here")\nprint("Content-type: text/html")\nprint()\nprint("<pre>")\nfor k, v in os.environ.items():\n    try:\n        k.encode(\'ascii\')\n        v.encode(\'ascii\')\n    except UnicodeEncodeError:\n        continue  # see: BPO-44647\n    print(f"{k}={v}")\nprint("</pre>")\n'

class SocketlessRequestHandler(SimpleHTTPRequestHandler):

    def __init__(self, directory=None):
        request = mock.Mock()
        request.makefile.return_value = BytesIO()
        super().__init__(request, None, None, directory=directory)
        self.get_called = False
        self.protocol_version = 'HTTP/1.1'

    def do_GET(self):
        self.get_called = True
        self.send_response(HTTPStatus.OK)
        self.send_header('Content-Type', 'text/html')
        self.end_headers()
        self.wfile.write(b'<html><body>Data</body></html>\r\n')

    def log_message(self, format, *args):
        pass

class RejectingSocketlessRequestHandler(SocketlessRequestHandler):

    def handle_expect_100(self):
        self.send_error(HTTPStatus.EXPECTATION_FAILED)
        return False

class AuditableBytesIO:

    def __init__(self):
        self.datas = []

    def write(self, data):
        self.datas.append(data)

    def getData(self):
        return b''.join(self.datas)

    @property
    def numWrites(self):
        return len(self.datas)

def setUpModule():
    unittest.addModuleCleanup(os.chdir, os.getcwd())


# --- test body ---
HTTPResponseMatch = re.compile(b'HTTP/1.[0-9]+ 200 OK')

def send_typical_request(message):
    input = BytesIO(message)
    output = BytesIO()
    self_handler.rfile = input
    self_handler.wfile = output
    self_handler.handle_one_request()
    output.seek(0)
    return output.readlines()

def verify_expected_headers(headers):
    for fieldName in (b'Server: ', b'Date: ', b'Content-Type: '):

        assert sum((h.startswith(fieldName) for h in headers)) == 1

def verify_get_called():

    assert self_handler.get_called

def verify_http_server_response(response):
    match = HTTPResponseMatch.search(response)

    assert match is not None
self_handler = SocketlessRequestHandler()
result = send_typical_request(b'GET / HTTP/1.1\r\n' + b'X-Foo: bar\r\n' * 101 + b'\r\n')

assert result[0] == b'HTTP/1.1 431 Too many headers\r\n'

assert not self_handler.get_called

assert self_handler.requestline == 'GET / HTTP/1.1'
print("BaseHTTPRequestHandlerTestCase::test_too_many_headers: ok")
