# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "urllib2"
# dimension = "behavior"
# case = "handler_tests__test_http_body_fileobj"
# subject = "cpython.test_urllib2.HandlerTests.test_http_body_fileobj"
# kind = "semantic"
# xfail = "auto-ported CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_urllib2.py"
# status = "filled"
# ///
# mamba-xfail: auto-ported CPython test; mamba promotion pending
# Auto-ported from CPython 3.12 test_urllib2.py::HandlerTests::test_http_body_fileobj
"""Auto-ported test: HandlerTests::test_http_body_fileobj (CPython 3.12 oracle)."""


import unittest
from test import support
from test.support import os_helper
from test.support import warnings_helper
from test import test_urllib
from unittest import mock
import os
import io
import socket
import array
import sys
import tempfile
import subprocess
import urllib.request
from urllib.request import Request, OpenerDirector, HTTPBasicAuthHandler, HTTPPasswordMgrWithPriorAuth, _parse_proxy, _proxy_bypass_winreg_override, _proxy_bypass_macosx_sysconf, AbstractDigestAuthHandler
from urllib.parse import urlparse
import urllib.error
import http.client


support.requires_working_socket(module=True)

class MockOpener:
    addheaders = []

    def open(self, req, data=None, timeout=socket._GLOBAL_DEFAULT_TIMEOUT):
        self.req, self.data, self.timeout = (req, data, timeout)

    def error(self, proto, *args):
        self.proto, self.args = (proto, args)

class MockFile:

    def read(self, count=None):
        pass

    def readline(self, count=None):
        pass

    def close(self):
        pass

class MockHeaders(dict):

    def getheaders(self, name):
        return list(self.values())

class MockResponse(io.StringIO):

    def __init__(self, code, msg, headers, data, url=None):
        io.StringIO.__init__(self, data)
        self.code, self.msg, self.headers, self.url = (code, msg, headers, url)

    def info(self):
        return self.headers

    def geturl(self):
        return self.url

class MockCookieJar:

    def add_cookie_header(self, request):
        self.ach_req = request

    def extract_cookies(self, response, request):
        self.ec_req, self.ec_r = (request, response)

class FakeMethod:

    def __init__(self, meth_name, action, handle):
        self.meth_name = meth_name
        self.handle = handle
        self.action = action

    def __call__(self, *args):
        return self.handle(self.meth_name, self.action, *args)

class MockHTTPResponse(io.IOBase):

    def __init__(self, fp, msg, status, reason):
        self.fp = fp
        self.msg = msg
        self.status = status
        self.reason = reason
        self.code = 200

    def read(self):
        return ''

    def info(self):
        return {}

    def geturl(self):
        return self.url

class MockHTTPClass:

    def __init__(self):
        self.level = 0
        self.req_headers = []
        self.data = None
        self.raise_on_endheaders = False
        self.sock = None
        self._tunnel_headers = {}

    def __call__(self, host, timeout=socket._GLOBAL_DEFAULT_TIMEOUT):
        self.host = host
        self.timeout = timeout
        return self

    def set_debuglevel(self, level):
        self.level = level

    def set_tunnel(self, host, port=None, headers=None):
        self._tunnel_host = host
        self._tunnel_port = port
        if headers:
            self._tunnel_headers = headers
        else:
            self._tunnel_headers.clear()

    def request(self, method, url, body=None, headers=None, *, encode_chunked=False):
        self.method = method
        self.selector = url
        if headers is not None:
            self.req_headers += headers.items()
        self.req_headers.sort()
        if body:
            self.data = body
        self.encode_chunked = encode_chunked
        if self.raise_on_endheaders:
            raise OSError()

    def getresponse(self):
        return MockHTTPResponse(MockFile(), {}, 200, 'OK')

    def close(self):
        pass

class MockHandler:
    handler_order = 500

    def __init__(self, methods):
        self._define_methods(methods)

    def _define_methods(self, methods):
        for spec in methods:
            if len(spec) == 2:
                name, action = spec
            else:
                name, action = (spec, None)
            meth = FakeMethod(name, action, self.handle)
            setattr(self.__class__, name, meth)

    def handle(self, fn_name, action, *args, **kwds):
        self.parent.calls.append((self, fn_name, args, kwds))
        if action is None:
            return None
        elif action == 'return self':
            return self
        elif action == 'return response':
            res = MockResponse(200, 'OK', {}, '')
            return res
        elif action == 'return request':
            return Request('http://blah/')
        elif action.startswith('error'):
            code = action[action.rfind(' ') + 1:]
            try:
                code = int(code)
            except ValueError:
                pass
            res = MockResponse(200, 'OK', {}, '')
            return self.parent.error('http', args[0], res, code, '', {})
        elif action == 'raise':
            raise urllib.error.URLError('blah')
        assert False

    def close(self):
        pass

    def add_parent(self, parent):
        self.parent = parent
        self.parent.calls = []

    def __lt__(self, other):
        if not hasattr(other, 'handler_order'):
            return True
        return self.handler_order < other.handler_order

def add_ordered_mock_handlers(opener, meth_spec):
    """Create MockHandlers and add them to an OpenerDirector.

    meth_spec: list of lists of tuples and strings defining methods to define
    on handlers.  eg:

    [["http_error", "ftp_open"], ["http_open"]]

    defines methods .http_error() and .ftp_open() on one handler, and
    .http_open() on another.  These methods just record their arguments and
    return None.  Using a tuple instead of a string causes the method to
    perform some action (see MockHandler.handle()), eg:

    [["http_error"], [("http_open", "return request")]]

    defines .http_error() on one handler (which simply returns None), and
    .http_open() on another handler, which returns a Request object.

    """
    handlers = []
    count = 0
    for meths in meth_spec:

        class MockHandlerSubclass(MockHandler):
            pass
        h = MockHandlerSubclass(meths)
        h.handler_order += count
        h.add_parent(opener)
        count = count + 1
        handlers.append(h)
        opener.add_handler(h)
    return handlers

def build_test_opener(*handler_instances):
    opener = OpenerDirector()
    for h in handler_instances:
        opener.add_handler(h)
    return opener

class MockHTTPHandler(urllib.request.HTTPHandler):

    def __init__(self, debuglevel=None):
        super(MockHTTPHandler, self).__init__(debuglevel=debuglevel)
        self.httpconn = MockHTTPClass()

    def http_open(self, req):
        return self.do_open(self.httpconn, req)

class MockHTTPHandlerRedirect(urllib.request.BaseHandler):

    def __init__(self, code, headers):
        self.code = code
        self.headers = headers
        self.reset()

    def reset(self):
        self._count = 0
        self.requests = []

    def http_open(self, req):
        import email, copy
        self.requests.append(copy.deepcopy(req))
        if self._count == 0:
            self._count = self._count + 1
            name = http.client.responses[self.code]
            msg = email.message_from_string(self.headers)
            return self.parent.error('http', req, MockFile(), self.code, name, msg)
        else:
            self.req = req
            msg = email.message_from_string('\r\n\r\n')
            return MockResponse(200, 'OK', msg, '', req.get_full_url())

if hasattr(http.client, 'HTTPSConnection'):

    class MockHTTPSHandler(urllib.request.HTTPSHandler):

        def __init__(self, debuglevel=None, context=None, check_hostname=None):
            super(MockHTTPSHandler, self).__init__(debuglevel, context, check_hostname)
            self.httpconn = MockHTTPClass()

        def https_open(self, req):
            return self.do_open(self.httpconn, req)

class MockHTTPHandlerCheckAuth(urllib.request.BaseHandler):

    def __init__(self, code):
        self.code = code
        self.has_auth_header = False

    def reset(self):
        self.has_auth_header = False

    def http_open(self, req):
        if req.has_header('Authorization'):
            self.has_auth_header = True
        name = http.client.responses[self.code]
        return MockResponse(self.code, name, MockFile(), '', req.get_full_url())

class MockPasswordManager:

    def add_password(self, realm, uri, user, password):
        self.realm = realm
        self.url = uri
        self.user = user
        self.password = password

    def find_user_password(self, realm, authuri):
        self.target_realm = realm
        self.target_url = authuri
        return (self.user, self.password)

def sanepathname2url(path):
    urlpath = urllib.request.pathname2url(path)
    if os.name == 'nt' and urlpath.startswith('///'):
        urlpath = urlpath[2:]
    return urlpath


# --- test body ---
h = urllib.request.AbstractHTTPHandler()
o = h.parent = MockOpener()
file_obj = io.BytesIO()
req = Request('http://example.com/', file_obj, {})
newreq = h.do_request_(req)

assert newreq.get_header('Transfer-encoding') == 'chunked'

assert not newreq.has_header('Content-length')
headers = {'Content-Length': 30}
req = Request('http://example.com/', file_obj, headers)
newreq = h.do_request_(req)

assert int(newreq.get_header('Content-length')) == 30

assert not newreq.has_header('Transfer-encoding')
file_obj.close()
print("HandlerTests::test_http_body_fileobj: ok")
