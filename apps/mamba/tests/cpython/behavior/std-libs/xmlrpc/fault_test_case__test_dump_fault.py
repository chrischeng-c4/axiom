# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "xmlrpc"
# dimension = "behavior"
# case = "fault_test_case__test_dump_fault"
# subject = "cpython.test_xmlrpc.FaultTestCase.test_dump_fault"
# kind = "semantic"
# xfail = "auto-ported CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_xmlrpc.py"
# status = "filled"
# ///
# mamba-xfail: auto-ported CPython test; mamba promotion pending
# Auto-ported from CPython 3.12 test_xmlrpc.py::FaultTestCase::test_dump_fault
"""Auto-ported test: FaultTestCase::test_dump_fault (CPython 3.12 oracle)."""


import base64
import datetime
import decimal
import sys
import time
import unittest
from unittest import mock
import xmlrpc.client as xmlrpclib
import xmlrpc.server
import http.client
import http, http.server
import socket
import threading
import re
import io
import contextlib
from test import support
from test.support import os_helper
from test.support import socket_helper
from test.support import threading_helper
from test.support import ALWAYS_EQ, LARGEST, SMALLEST


try:
    import gzip
except ImportError:
    gzip = None

support.requires_working_socket(module=True)

alist = [{'astring': 'foo@bar.baz.spam', 'afloat': 7283.43, 'anint': 2 ** 20, 'ashortlong': 2, 'anotherlist': ['.zyx.41'], 'abase64': xmlrpclib.Binary(b'my dog has fleas'), 'b64bytes': b'my dog has fleas', 'b64bytearray': bytearray(b'my dog has fleas'), 'boolean': False, 'unicode': '䀀怀耀', 'ukey䀀': 'regular value', 'datetime1': xmlrpclib.DateTime('20050210T11:41:23'), 'datetime2': xmlrpclib.DateTime((2005, 2, 10, 11, 41, 23, 0, 1, -1)), 'datetime3': xmlrpclib.DateTime(datetime.datetime(2005, 2, 10, 11, 41, 23))}]

ADDR = PORT = URL = None

def http_server(evt, numrequests, requestHandler=None, encoding=None):

    class TestInstanceClass:

        def div(self, x, y):
            return x // y

        def _methodHelp(self, name):
            if name == 'div':
                return 'This is the div function'

        class Fixture:

            @staticmethod
            def getData():
                return '42'

    class MyXMLRPCServer(xmlrpc.server.SimpleXMLRPCServer):

        def get_request(self):
            s, port = self.socket.accept()
            s.setblocking(True)
            return (s, port)
    if not requestHandler:
        requestHandler = xmlrpc.server.SimpleXMLRPCRequestHandler
    serv = MyXMLRPCServer(('localhost', 0), requestHandler, encoding=encoding, logRequests=False, bind_and_activate=False)
    try:
        serv.server_bind()
        global ADDR, PORT, URL
        ADDR, PORT = serv.socket.getsockname()
        URL = 'http://%s:%d' % (ADDR, PORT)
        serv.server_activate()
        serv.register_introspection_functions()
        serv.register_multicall_functions()
        serv.register_function(pow)
        serv.register_function(lambda x: x, 'têšt')

        @serv.register_function
        def my_function():
            """This is my function"""
            return True

        @serv.register_function(name='add')
        def _(x, y):
            return x + y
        testInstance = TestInstanceClass()
        serv.register_instance(testInstance, allow_dotted_names=True)
        evt.set()
        while numrequests > 0:
            serv.handle_request()
            numrequests -= 1
    except TimeoutError:
        pass
    finally:
        serv.socket.close()
        PORT = None
        evt.set()

def http_multi_server(evt, numrequests, requestHandler=None):

    class TestInstanceClass:

        def div(self, x, y):
            return x // y

        def _methodHelp(self, name):
            if name == 'div':
                return 'This is the div function'

    def my_function():
        """This is my function"""
        return True

    class MyXMLRPCServer(xmlrpc.server.MultiPathXMLRPCServer):

        def get_request(self):
            s, port = self.socket.accept()
            s.setblocking(True)
            return (s, port)
    if not requestHandler:
        requestHandler = xmlrpc.server.SimpleXMLRPCRequestHandler

    class MyRequestHandler(requestHandler):
        rpc_paths = []

    class BrokenDispatcher:

        def _marshaled_dispatch(self, data, dispatch_method=None, path=None):
            raise RuntimeError('broken dispatcher')
    serv = MyXMLRPCServer(('localhost', 0), MyRequestHandler, logRequests=False, bind_and_activate=False)
    serv.socket.settimeout(3)
    serv.server_bind()
    try:
        global ADDR, PORT, URL
        ADDR, PORT = serv.socket.getsockname()
        URL = 'http://%s:%d' % (ADDR, PORT)
        serv.server_activate()
        paths = ['/foo', '/foo/bar', '/foo?k=v', '/foo#frag', '/foo?k=v#frag', '', '/', '/RPC2', '?k=v', '#frag']
        for path in paths:
            d = serv.add_dispatcher(path, xmlrpc.server.SimpleXMLRPCDispatcher())
            d.register_introspection_functions()
            d.register_multicall_functions()
            d.register_function(lambda p=path: p, 'test')
        serv.get_dispatcher(paths[0]).register_function(pow)
        serv.get_dispatcher(paths[1]).register_function(lambda x, y: x + y, 'add')
        serv.add_dispatcher('/is/broken', BrokenDispatcher())
        evt.set()
        while numrequests > 0:
            serv.handle_request()
            numrequests -= 1
    except TimeoutError:
        pass
    finally:
        serv.socket.close()
        PORT = None
        evt.set()

def is_unavailable_exception(e):
    """Returns True if the given ProtocolError is the product of a server-side
       exception caused by the 'temporarily unavailable' response sometimes
       given by operations on non-blocking sockets."""
    try:
        if e.errcode == -1 or e.headers is None:
            return True
        exc_mess = e.headers.get('X-exception')
    except AttributeError:
        exc_mess = str(e)
    if exc_mess and 'temporarily unavailable' in exc_mess.lower():
        return True

def make_request_and_skipIf(condition, reason):
    if not condition:
        return lambda func: func

    def decorator(func):

        def make_request_and_skip(self):
            try:
                xmlrpclib.ServerProxy(URL).my_function()
            except (xmlrpclib.ProtocolError, OSError) as e:
                if not is_unavailable_exception(e):
                    raise
            raise unittest.SkipTest(reason)
        return make_request_and_skip
    return decorator

class BaseServerTestCase(unittest.TestCase):
    requestHandler = None
    request_count = 1
    threadFunc = staticmethod(http_server)

    def setUp(self):
        xmlrpc.server.SimpleXMLRPCServer._send_traceback_header = True
        self.evt = threading.Event()
        serv_args = (self.evt, self.request_count, self.requestHandler)
        thread = threading.Thread(target=self.threadFunc, args=serv_args)
        thread.start()
        self.addCleanup(thread.join)
        self.evt.wait()
        self.evt.clear()

    def tearDown(self):
        self.evt.wait()
        xmlrpc.server.SimpleXMLRPCServer._send_traceback_header = False

class BaseKeepaliveServerTestCase(BaseServerTestCase):

    class RequestHandler(xmlrpc.server.SimpleXMLRPCRequestHandler):
        parentClass = xmlrpc.server.SimpleXMLRPCRequestHandler
        protocol_version = 'HTTP/1.1'
        myRequests = []

        def handle(self):
            self.myRequests.append([])
            self.reqidx = len(self.myRequests) - 1
            return self.parentClass.handle(self)

        def handle_one_request(self):
            result = self.parentClass.handle_one_request(self)
            self.myRequests[self.reqidx].append(self.raw_requestline)
            return result
    requestHandler = RequestHandler

    def setUp(self):
        self.RequestHandler.myRequests = []
        return BaseServerTestCase.setUp(self)

class FailingMessageClass(http.client.HTTPMessage):

    def get(self, key, failobj=None):
        key = key.lower()
        if key == 'content-length':
            return 'I am broken'
        return super().get(key, failobj)

@contextlib.contextmanager
def captured_stdout(encoding='utf-8'):
    """A variation on support.captured_stdout() which gives a text stream
    having a `buffer` attribute.
    """
    orig_stdout = sys.stdout
    sys.stdout = io.TextIOWrapper(io.BytesIO(), encoding=encoding)
    try:
        yield sys.stdout
    finally:
        sys.stdout = orig_stdout

def setUpModule():
    thread_info = threading_helper.threading_setup()
    unittest.addModuleCleanup(threading_helper.threading_cleanup, *thread_info)


# --- test body ---
f = xmlrpclib.Fault(42, 'Test Fault')
s = xmlrpclib.dumps((f,))
(newf,), m = xmlrpclib.loads(s)

assert newf == {'faultCode': 42, 'faultString': 'Test Fault'}

assert m == None
s = xmlrpclib.Marshaller().dumps(f)

try:
    xmlrpclib.loads(s)
    raise AssertionError('expected xmlrpclib.Fault')
except xmlrpclib.Fault:
    pass
print("FaultTestCase::test_dump_fault: ok")
