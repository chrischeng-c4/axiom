# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "http"
# dimension = "behavior"
# case = "basic_test__test_simple_httpstatus"
# subject = "cpython.test_httplib.BasicTest.test_simple_httpstatus"
# kind = "semantic"
# xfail = "auto-ported CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_httplib.py"
# status = "filled"
# ///
# mamba-xfail: auto-ported CPython test; mamba promotion pending
# Auto-ported from CPython 3.12 test_httplib.py::BasicTest::test_simple_httpstatus
"""Auto-ported test: BasicTest::test_simple_httpstatus (CPython 3.12 oracle)."""


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
class CheckedHTTPStatus(enum.IntEnum):
    """HTTP status codes and reason phrases

            Status codes from the following RFCs are all observed:

                * RFC 7231: Hypertext Transfer Protocol (HTTP/1.1), obsoletes 2616
                * RFC 6585: Additional HTTP Status Codes
                * RFC 3229: Delta encoding in HTTP
                * RFC 4918: HTTP Extensions for WebDAV, obsoletes 2518
                * RFC 5842: Binding Extensions to WebDAV
                * RFC 7238: Permanent Redirect
                * RFC 2295: Transparent Content Negotiation in HTTP
                * RFC 2774: An HTTP Extension Framework
                * RFC 7725: An HTTP Status Code to Report Legal Obstacles
                * RFC 7540: Hypertext Transfer Protocol Version 2 (HTTP/2)
                * RFC 2324: Hyper Text Coffee Pot Control Protocol (HTCPCP/1.0)
                * RFC 8297: An HTTP Status Code for Indicating Hints
                * RFC 8470: Using Early Data in HTTP
            """

    def __new__(cls, value, phrase, description=''):
        obj = int.__new__(cls, value)
        obj._value_ = value
        obj.phrase = phrase
        obj.description = description
        return obj

    @property
    def is_informational(self):
        return 100 <= self <= 199

    @property
    def is_success(self):
        return 200 <= self <= 299

    @property
    def is_redirection(self):
        return 300 <= self <= 399

    @property
    def is_client_error(self):
        return 400 <= self <= 499

    @property
    def is_server_error(self):
        return 500 <= self <= 599
    CONTINUE = (100, 'Continue', 'Request received, please continue')
    SWITCHING_PROTOCOLS = (101, 'Switching Protocols', 'Switching to new protocol; obey Upgrade header')
    PROCESSING = (102, 'Processing')
    EARLY_HINTS = (103, 'Early Hints')
    OK = (200, 'OK', 'Request fulfilled, document follows')
    CREATED = (201, 'Created', 'Document created, URL follows')
    ACCEPTED = (202, 'Accepted', 'Request accepted, processing continues off-line')
    NON_AUTHORITATIVE_INFORMATION = (203, 'Non-Authoritative Information', 'Request fulfilled from cache')
    NO_CONTENT = (204, 'No Content', 'Request fulfilled, nothing follows')
    RESET_CONTENT = (205, 'Reset Content', 'Clear input form for further input')
    PARTIAL_CONTENT = (206, 'Partial Content', 'Partial content follows')
    MULTI_STATUS = (207, 'Multi-Status')
    ALREADY_REPORTED = (208, 'Already Reported')
    IM_USED = (226, 'IM Used')
    MULTIPLE_CHOICES = (300, 'Multiple Choices', 'Object has several resources -- see URI list')
    MOVED_PERMANENTLY = (301, 'Moved Permanently', 'Object moved permanently -- see URI list')
    FOUND = (302, 'Found', 'Object moved temporarily -- see URI list')
    SEE_OTHER = (303, 'See Other', 'Object moved -- see Method and URL list')
    NOT_MODIFIED = (304, 'Not Modified', 'Document has not changed since given time')
    USE_PROXY = (305, 'Use Proxy', 'You must use proxy specified in Location to access this resource')
    TEMPORARY_REDIRECT = (307, 'Temporary Redirect', 'Object moved temporarily -- see URI list')
    PERMANENT_REDIRECT = (308, 'Permanent Redirect', 'Object moved permanently -- see URI list')
    BAD_REQUEST = (400, 'Bad Request', 'Bad request syntax or unsupported method')
    UNAUTHORIZED = (401, 'Unauthorized', 'No permission -- see authorization schemes')
    PAYMENT_REQUIRED = (402, 'Payment Required', 'No payment -- see charging schemes')
    FORBIDDEN = (403, 'Forbidden', 'Request forbidden -- authorization will not help')
    NOT_FOUND = (404, 'Not Found', 'Nothing matches the given URI')
    METHOD_NOT_ALLOWED = (405, 'Method Not Allowed', 'Specified method is invalid for this resource')
    NOT_ACCEPTABLE = (406, 'Not Acceptable', 'URI not available in preferred format')
    PROXY_AUTHENTICATION_REQUIRED = (407, 'Proxy Authentication Required', 'You must authenticate with this proxy before proceeding')
    REQUEST_TIMEOUT = (408, 'Request Timeout', 'Request timed out; try again later')
    CONFLICT = (409, 'Conflict', 'Request conflict')
    GONE = (410, 'Gone', 'URI no longer exists and has been permanently removed')
    LENGTH_REQUIRED = (411, 'Length Required', 'Client must specify Content-Length')
    PRECONDITION_FAILED = (412, 'Precondition Failed', 'Precondition in headers is false')
    REQUEST_ENTITY_TOO_LARGE = (413, 'Request Entity Too Large', 'Entity is too large')
    REQUEST_URI_TOO_LONG = (414, 'Request-URI Too Long', 'URI is too long')
    UNSUPPORTED_MEDIA_TYPE = (415, 'Unsupported Media Type', 'Entity body in unsupported format')
    REQUESTED_RANGE_NOT_SATISFIABLE = (416, 'Requested Range Not Satisfiable', 'Cannot satisfy request range')
    EXPECTATION_FAILED = (417, 'Expectation Failed', 'Expect condition could not be satisfied')
    IM_A_TEAPOT = (418, "I'm a Teapot", 'Server refuses to brew coffee because it is a teapot.')
    MISDIRECTED_REQUEST = (421, 'Misdirected Request', 'Server is not able to produce a response')
    UNPROCESSABLE_ENTITY = (422, 'Unprocessable Entity')
    LOCKED = (423, 'Locked')
    FAILED_DEPENDENCY = (424, 'Failed Dependency')
    TOO_EARLY = (425, 'Too Early')
    UPGRADE_REQUIRED = (426, 'Upgrade Required')
    PRECONDITION_REQUIRED = (428, 'Precondition Required', 'The origin server requires the request to be conditional')
    TOO_MANY_REQUESTS = (429, 'Too Many Requests', 'The user has sent too many requests in a given amount of time ("rate limiting")')
    REQUEST_HEADER_FIELDS_TOO_LARGE = (431, 'Request Header Fields Too Large', 'The server is unwilling to process the request because its header fields are too large')
    UNAVAILABLE_FOR_LEGAL_REASONS = (451, 'Unavailable For Legal Reasons', 'The server is denying access to the resource as a consequence of a legal demand')
    INTERNAL_SERVER_ERROR = (500, 'Internal Server Error', 'Server got itself in trouble')
    NOT_IMPLEMENTED = (501, 'Not Implemented', 'Server does not support this operation')
    BAD_GATEWAY = (502, 'Bad Gateway', 'Invalid responses from another server/proxy')
    SERVICE_UNAVAILABLE = (503, 'Service Unavailable', 'The server cannot process the request due to a high load')
    GATEWAY_TIMEOUT = (504, 'Gateway Timeout', 'The gateway server did not receive a timely response')
    HTTP_VERSION_NOT_SUPPORTED = (505, 'HTTP Version Not Supported', 'Cannot fulfill request')
    VARIANT_ALSO_NEGOTIATES = (506, 'Variant Also Negotiates')
    INSUFFICIENT_STORAGE = (507, 'Insufficient Storage')
    LOOP_DETECTED = (508, 'Loop Detected')
    NOT_EXTENDED = (510, 'Not Extended')
    NETWORK_AUTHENTICATION_REQUIRED = (511, 'Network Authentication Required', 'The client needs to authenticate to gain network access')
enum._test_simple_enum(CheckedHTTPStatus, HTTPStatus)
print("BasicTest::test_simple_httpstatus: ok")
