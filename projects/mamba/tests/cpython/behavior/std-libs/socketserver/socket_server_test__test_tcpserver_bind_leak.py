# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "socketserver"
# dimension = "behavior"
# case = "socket_server_test__test_tcpserver_bind_leak"
# subject = "cpython.test_socketserver.SocketServerTest.test_tcpserver_bind_leak"
# kind = "semantic"
# xfail = "auto-ported CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_socketserver.py"
# status = "filled"
# ///
# mamba-xfail: auto-ported CPython test; mamba promotion pending
# Auto-ported from CPython 3.12 test_socketserver.py::SocketServerTest::test_tcpserver_bind_leak
"""Auto-ported test: SocketServerTest::test_tcpserver_bind_leak (CPython 3.12 oracle)."""


import contextlib
import io
import os
import select
import signal
import socket
import threading
import unittest
import socketserver
import test.support
from test.support import reap_children, verbose
from test.support import os_helper
from test.support import socket_helper
from test.support import threading_helper


'\nTest suite for socketserver.\n'

test.support.requires('network')

test.support.requires_working_socket(module=True)

TEST_STR = b'hello world\n'

HOST = socket_helper.HOST

HAVE_UNIX_SOCKETS = hasattr(socket, 'AF_UNIX')

requires_unix_sockets = unittest.skipUnless(HAVE_UNIX_SOCKETS, 'requires Unix sockets')

HAVE_FORKING = test.support.has_fork_support

requires_forking = unittest.skipUnless(HAVE_FORKING, 'requires forking')

_real_select = select.select

def receive(sock, n, timeout=test.support.SHORT_TIMEOUT):
    r, w, x = _real_select([sock], [], [], timeout)
    if sock in r:
        return sock.recv(n)
    else:
        raise RuntimeError('timed out on %r' % (sock,))

@test.support.requires_fork()
@contextlib.contextmanager
def simple_subprocess(testcase):
    """Tests that a custom child process is not waited on (Issue 1540386)"""
    pid = os.fork()
    if pid == 0:
        os._exit(72)
    try:
        yield None
    except:
        raise
    finally:
        test.support.wait_process(pid, exitcode=72)

class BaseErrorTestServer(socketserver.TCPServer):

    def __init__(self, exception):
        self.exception = exception
        super().__init__((HOST, 0), BadHandler)
        with socket.create_connection(self.server_address):
            pass
        try:
            self.handle_request()
        finally:
            self.server_close()
        self.wait_done()

    def handle_error(self, request, client_address):
        with open(os_helper.TESTFN, 'a') as log:
            log.write('Error handled\n')

    def wait_done(self):
        pass

class BadHandler(socketserver.BaseRequestHandler):

    def handle(self):
        with open(os_helper.TESTFN, 'a') as log:
            log.write('Handler called\n')
        raise self.server.exception('Test error')

class ThreadingErrorTestServer(socketserver.ThreadingMixIn, BaseErrorTestServer):

    def __init__(self, *pos, **kw):
        self.done = threading.Event()
        super().__init__(*pos, **kw)

    def shutdown_request(self, *pos, **kw):
        super().shutdown_request(*pos, **kw)
        self.done.set()

    def wait_done(self):
        self.done.wait()

if HAVE_FORKING:

    class ForkingErrorTestServer(socketserver.ForkingMixIn, BaseErrorTestServer):
        pass


# --- test body ---
self_port_seed = 0
self_test_files = []
for i in range(1024):
    try:
        socketserver.TCPServer((HOST, -1), socketserver.StreamRequestHandler)
        raise AssertionError('expected OverflowError')
    except OverflowError:
        pass
print("SocketServerTest::test_tcpserver_bind_leak: ok")
