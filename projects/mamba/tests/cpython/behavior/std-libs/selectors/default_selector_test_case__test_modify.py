# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "selectors"
# dimension = "behavior"
# case = "default_selector_test_case__test_modify"
# subject = "cpython.test_selectors.DefaultSelectorTestCase.test_modify"
# kind = "semantic"
# xfail = "auto-ported CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_selectors.py"
# status = "filled"
# ///
# mamba-xfail: auto-ported CPython test; mamba promotion pending
# Auto-ported from CPython 3.12 test_selectors.py::DefaultSelectorTestCase::test_modify
"""Auto-ported test: DefaultSelectorTestCase::test_modify (CPython 3.12 oracle)."""


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
rd, wr = make_socketpair()
key = s.register(rd, selectors.EVENT_READ)
key2 = s.modify(rd, selectors.EVENT_WRITE)

assert key.events != key2.events

assert key2 == s.get_key(rd)
s.unregister(rd)
d1 = object()
d2 = object()
key = s.register(rd, selectors.EVENT_READ, d1)
key2 = s.modify(rd, selectors.EVENT_READ, d2)

assert key.events == key2.events

assert key.data != key2.data

assert key2 == s.get_key(rd)

assert key2.data == d2

try:
    s.modify(999999, selectors.EVENT_READ)
    raise AssertionError('expected KeyError')
except KeyError:
    pass
d3 = object()
s.register = unittest.mock.Mock()
s.unregister = unittest.mock.Mock()
s.modify(rd, selectors.EVENT_READ, d3)

assert not s.register.called

assert not s.unregister.called
print("DefaultSelectorTestCase::test_modify: ok")
