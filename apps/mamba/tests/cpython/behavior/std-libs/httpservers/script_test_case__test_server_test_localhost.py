# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "httpservers"
# dimension = "behavior"
# case = "script_test_case__test_server_test_localhost"
# subject = "cpython.test_httpservers.ScriptTestCase.test_server_test_localhost"
# kind = "semantic"
# xfail = "auto-ported CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_httpservers.py"
# status = "filled"
# ///
# mamba-xfail: auto-ported CPython test; mamba promotion pending
# Auto-ported from CPython 3.12 test_httpservers.py::ScriptTestCase::test_server_test_localhost
"""Auto-ported test: ScriptTestCase::test_server_test_localhost (CPython 3.12 oracle)."""


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
ipv6_addrs = ('::', '2001:0db8:85a3:0000:0000:8a2e:0370:7334', '::1')
ipv4_addrs = ('0.0.0.0', '8.8.8.8', '127.0.0.1')

def mock_server_class():
    return mock.MagicMock(return_value=mock.MagicMock(__enter__=mock.MagicMock(return_value=mock.MagicMock(socket=mock.MagicMock(getsockname=lambda: ('', 0))))))
mock_server = mock_server_class()
server.test(ServerClass=mock_server, bind='localhost')

assert mock_server.address_family in (socket.AF_INET6, socket.AF_INET)
print("ScriptTestCase::test_server_test_localhost: ok")
