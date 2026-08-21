# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "telnetlib"
# dimension = "behavior"
# case = "read_tests__test_read_some"
# subject = "cpython.test_telnetlib.ReadTests.test_read_some"
# kind = "semantic"
# xfail = "auto-ported CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_telnetlib.py"
# status = "filled"
# ///
# mamba-xfail: auto-ported CPython test; mamba promotion pending
# Auto-ported from CPython 3.12 test_telnetlib.py::ReadTests::test_read_some
"""Auto-ported test: ReadTests::test_read_some (CPython 3.12 oracle)."""


import socket
import selectors
import threading
import contextlib
from test import support
from test.support import socket_helper, warnings_helper
import unittest


support.requires_working_socket(module=True)

telnetlib = warnings_helper.import_deprecated('telnetlib')

HOST = socket_helper.HOST

def server(evt, serv):
    serv.listen()
    evt.set()
    try:
        conn, addr = serv.accept()
        conn.close()
    except TimeoutError:
        pass
    finally:
        serv.close()

class GeneralTests(unittest.TestCase):

    def setUp(self):
        self.evt = threading.Event()
        self.sock = socket.socket(socket.AF_INET, socket.SOCK_STREAM)
        self.sock.settimeout(60)
        self.port = socket_helper.bind_port(self.sock)
        self.thread = threading.Thread(target=server, args=(self.evt, self.sock))
        self.thread.daemon = True
        self.thread.start()
        self.evt.wait()

    def tearDown(self):
        self.thread.join()
        del self.thread

    def testBasic(self):
        telnet = telnetlib.Telnet(HOST, self.port)
        telnet.sock.close()

    def testContextManager(self):
        with telnetlib.Telnet(HOST, self.port) as tn:
            self.assertIsNotNone(tn.get_socket())
        self.assertIsNone(tn.get_socket())

    def testTimeoutDefault(self):
        self.assertTrue(socket.getdefaulttimeout() is None)
        socket.setdefaulttimeout(30)
        try:
            telnet = telnetlib.Telnet(HOST, self.port)
        finally:
            socket.setdefaulttimeout(None)
        self.assertEqual(telnet.sock.gettimeout(), 30)
        telnet.sock.close()

    def testTimeoutNone(self):
        self.assertTrue(socket.getdefaulttimeout() is None)
        socket.setdefaulttimeout(30)
        try:
            telnet = telnetlib.Telnet(HOST, self.port, timeout=None)
        finally:
            socket.setdefaulttimeout(None)
        self.assertTrue(telnet.sock.gettimeout() is None)
        telnet.sock.close()

    def testTimeoutValue(self):
        telnet = telnetlib.Telnet(HOST, self.port, timeout=30)
        self.assertEqual(telnet.sock.gettimeout(), 30)
        telnet.sock.close()

    def testTimeoutOpen(self):
        telnet = telnetlib.Telnet()
        telnet.open(HOST, self.port, timeout=30)
        self.assertEqual(telnet.sock.gettimeout(), 30)
        telnet.sock.close()

    def testGetters(self):
        telnet = telnetlib.Telnet(HOST, self.port, timeout=30)
        t_sock = telnet.sock
        self.assertEqual(telnet.get_socket(), t_sock)
        self.assertEqual(telnet.fileno(), t_sock.fileno())
        telnet.sock.close()

class SocketStub(object):
    """ a socket proxy that re-defines sendall() """

    def __init__(self, reads=()):
        self.reads = list(reads)
        self.writes = []
        self.block = False

    def sendall(self, data):
        self.writes.append(data)

    def recv(self, size):
        out = b''
        while self.reads and len(out) < size:
            out += self.reads.pop(0)
        if len(out) > size:
            self.reads.insert(0, out[size:])
            out = out[:size]
        return out

class TelnetAlike(telnetlib.Telnet):

    def fileno(self):
        raise NotImplementedError()

    def close(self):
        pass

    def sock_avail(self):
        return not self.sock.block

    def msg(self, msg, *args):
        with support.captured_stdout() as out:
            telnetlib.Telnet.msg(self, msg, *args)
        self._messages += out.getvalue()
        return

class MockSelector(selectors.BaseSelector):

    def __init__(self):
        self.keys = {}

    @property
    def resolution(self):
        return 0.001

    def register(self, fileobj, events, data=None):
        key = selectors.SelectorKey(fileobj, 0, events, data)
        self.keys[fileobj] = key
        return key

    def unregister(self, fileobj):
        return self.keys.pop(fileobj)

    def select(self, timeout=None):
        block = False
        for fileobj in self.keys:
            if isinstance(fileobj, TelnetAlike):
                block = fileobj.sock.block
                break
        if block:
            return []
        else:
            return [(key, key.events) for key in self.keys.values()]

    def get_map(self):
        return self.keys

@contextlib.contextmanager
def test_socket(reads):

    def new_conn(*ignored):
        return SocketStub(reads)
    try:
        old_conn = socket.create_connection
        socket.create_connection = new_conn
        yield None
    finally:
        socket.create_connection = old_conn
    return

def test_telnet(reads=(), cls=TelnetAlike):
    """ return a telnetlib.Telnet object that uses a SocketStub with
        reads queued up to be read """
    for x in reads:
        assert type(x) is bytes, x
    with test_socket(reads):
        telnet = cls('dummy', 0)
        telnet._messages = ''
    return telnet

class ExpectAndReadTestCase(unittest.TestCase):

    def setUp(self):
        self.old_selector = telnetlib._TelnetSelector
        telnetlib._TelnetSelector = MockSelector

    def tearDown(self):
        telnetlib._TelnetSelector = self.old_selector

class nego_collector(object):

    def __init__(self, sb_getter=None):
        self.seen = b''
        self.sb_getter = sb_getter
        self.sb_seen = b''

    def do_nego(self, sock, cmd, opt):
        self.seen += cmd + opt
        if cmd == tl.SE and self.sb_getter:
            sb_data = self.sb_getter()
            self.sb_seen += sb_data

tl = telnetlib


# --- test body ---
self_old_selector = telnetlib._TelnetSelector
telnetlib._TelnetSelector = MockSelector
'\n        read_some()\n          Read at least one byte or EOF; may block.\n        '
telnet = test_telnet([b'x' * 500])
data = telnet.read_some()

assert len(data) >= 1
telnet = test_telnet()
data = telnet.read_some()

assert b'' == data
print("ReadTests::test_read_some: ok")
