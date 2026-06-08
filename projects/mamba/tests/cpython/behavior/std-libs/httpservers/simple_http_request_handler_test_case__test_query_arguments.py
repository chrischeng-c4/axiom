# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "httpservers"
# dimension = "behavior"
# case = "simple_http_request_handler_test_case__test_query_arguments"
# subject = "cpython.test_httpservers.SimpleHTTPRequestHandlerTestCase.test_query_arguments"
# kind = "semantic"
# xfail = "auto-ported CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_httpservers.py"
# status = "filled"
# ///
# mamba-xfail: auto-ported CPython test; mamba promotion pending
# Auto-ported from CPython 3.12 test_httpservers.py::SimpleHTTPRequestHandlerTestCase::test_query_arguments
"""Auto-ported test: SimpleHTTPRequestHandlerTestCase::test_query_arguments (CPython 3.12 oracle)."""


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

class BaseTestCase(unittest.TestCase):

    def setUp(self):
        self._threads = threading_helper.threading_setup()
        os.environ = os_helper.EnvironmentVarGuard()
        self.server_started = threading.Event()
        self.thread = TestServerThread(self, self.request_handler)
        self.thread.start()
        self.server_started.wait()

    def tearDown(self):
        self.thread.stop()
        self.thread = None
        os.environ.__exit__()
        threading_helper.threading_cleanup(*self._threads)

    def request(self, uri, method='GET', body=None, headers={}):
        self.connection = http.client.HTTPConnection(self.HOST, self.PORT)
        self.connection.request(method, uri, body, headers)
        return self.connection.getresponse()

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
self_translated_1 = os.path.join(os.getcwd(), 'filename')
self_translated_2 = os.path.join('foo', 'filename')
self_translated_3 = os.path.join('bar', 'filename')
self_handler_1 = SocketlessRequestHandler()
self_handler_2 = SocketlessRequestHandler(directory='foo')
self_handler_3 = SocketlessRequestHandler(directory=pathlib.PurePath('bar'))
path = self_handler_1.translate_path('/filename')

assert path == self_translated_1
path = self_handler_2.translate_path('/filename')

assert path == self_translated_2
path = self_handler_3.translate_path('/filename')

assert path == self_translated_3
path = self_handler_1.translate_path('/filename?foo=bar')

assert path == self_translated_1
path = self_handler_2.translate_path('/filename?foo=bar')

assert path == self_translated_2
path = self_handler_3.translate_path('/filename?foo=bar')

assert path == self_translated_3
path = self_handler_1.translate_path('/filename?a=b&spam=eggs#zot')

assert path == self_translated_1
path = self_handler_2.translate_path('/filename?a=b&spam=eggs#zot')

assert path == self_translated_2
path = self_handler_3.translate_path('/filename?a=b&spam=eggs#zot')

assert path == self_translated_3
print("SimpleHTTPRequestHandlerTestCase::test_query_arguments: ok")
