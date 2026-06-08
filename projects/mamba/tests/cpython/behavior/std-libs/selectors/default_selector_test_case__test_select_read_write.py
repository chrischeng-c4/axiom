# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "selectors"
# dimension = "behavior"
# case = "default_selector_test_case__test_select_read_write"
# subject = "cpython.test_selectors.DefaultSelectorTestCase.test_select_read_write"
# kind = "semantic"
# xfail = "auto-ported CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_selectors.py"
# status = "filled"
# ///
# mamba-xfail: auto-ported CPython test; mamba promotion pending
# Auto-ported from CPython 3.12 test_selectors.py::DefaultSelectorTestCase::test_select_read_write
"""Auto-ported test: DefaultSelectorTestCase::test_select_read_write (CPython 3.12 oracle)."""


import errno
import os
import random
import selectors
import signal
import socket
import sys
from test import support
from test.support import os_helper
from test.support import socket_helper
from time import sleep
import unittest
import unittest.mock
import tempfile
from time import monotonic as time


try:
    import resource
except ImportError:
    resource = None

if support.is_emscripten or support.is_wasi:
    raise unittest.SkipTest('Cannot create socketpair on Emscripten/WASI.')

if hasattr(socket, 'socketpair'):
    socketpair = socket.socketpair
else:

    def socketpair(family=socket.AF_INET, type=socket.SOCK_STREAM, proto=0):
        with socket.socket(family, type, proto) as l:
            l.bind((socket_helper.HOST, 0))
            l.listen()
            c = socket.socket(family, type, proto)
            try:
                c.connect(l.getsockname())
                caddr = c.getsockname()
                while True:
                    a, addr = l.accept()
                    if addr == caddr:
                        return (c, a)
                    a.close()
            except OSError:
                c.close()
                raise

def find_ready_matching(ready, flag):
    match = []
    for key, events in ready:
        if events & flag:
            match.append(key.fileobj)
    return match

def tearDownModule():
    support.reap_children()


# --- test body ---
SELECTOR = selectors.DefaultSelector

def make_socketpair():
    rd, wr = socketpair()
    pass
    pass
    return (rd, wr)
s = SELECTOR()
pass
sock1, sock2 = make_socketpair()
sock2.send(b'foo')
my_key = s.register(sock1, selectors.EVENT_READ | selectors.EVENT_WRITE)
seen_read, seen_write = (False, False)
result = s.select()

assert len(result) <= 2
for key, events in result:

    assert isinstance(key, selectors.SelectorKey)

    assert key == my_key

    assert not events & ~(selectors.EVENT_READ | selectors.EVENT_WRITE)
    if events & selectors.EVENT_READ:

        assert not seen_read
        seen_read = True
    if events & selectors.EVENT_WRITE:

        assert not seen_write
        seen_write = True

assert seen_read

assert seen_write
print("DefaultSelectorTestCase::test_select_read_write: ok")
