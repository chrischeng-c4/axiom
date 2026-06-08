# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "telnetlib"
# dimension = "behavior"
# case = "option_tests__test_iac_commands"
# subject = "cpython.test_telnetlib.OptionTests.test_IAC_commands"
# kind = "semantic"
# xfail = "auto-ported CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_telnetlib.py"
# status = "filled"
# ///
# mamba-xfail: auto-ported CPython test; mamba promotion pending
# Auto-ported from CPython 3.12 test_telnetlib.py::OptionTests::test_IAC_commands
"""Auto-ported test: OptionTests::test_IAC_commands (CPython 3.12 oracle)."""


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
cmds = [tl.AO, tl.AYT, tl.BRK, tl.EC, tl.EL, tl.GA, tl.IP, tl.NOP]

def _test_command(data):
    """ helper for testing IAC + cmd """
    telnet = test_telnet(data)
    data_len = len(b''.join(data))
    nego = nego_collector()
    telnet.set_option_negotiation_callback(nego.do_nego)
    txt = telnet.read_all()
    cmd = nego.seen

    assert len(cmd) > 0

    assert cmd[:1] in cmds

    assert cmd[1:2] == tl.NOOPT

    assert data_len == len(txt + cmd)
    nego.sb_getter = None
for cmd in cmds:
    _test_command([tl.IAC, cmd])
    _test_command([b'x' * 100, tl.IAC, cmd, b'y' * 100])
    _test_command([b'x' * 10, tl.IAC, cmd, b'y' * 10])
_test_command([tl.IAC + cmd for cmd in cmds])
print("OptionTests::test_IAC_commands: ok")
