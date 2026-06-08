# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "selectors"
# dimension = "behavior"
# case = "default_selector_test_case__test_timeout"
# subject = "cpython.test_selectors.DefaultSelectorTestCase.test_timeout"
# kind = "semantic"
# xfail = "auto-ported CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_selectors.py"
# status = "filled"
# ///
# mamba-xfail: auto-ported CPython test; mamba promotion pending
# Auto-ported from CPython 3.12 test_selectors.py::DefaultSelectorTestCase::test_timeout
"""Auto-ported test: DefaultSelectorTestCase::test_timeout (CPython 3.12 oracle)."""


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
s.register(wr, selectors.EVENT_WRITE)
t = time()

assert 1 == len(s.select(0))

assert 1 == len(s.select(-1))

assert time() - t < 0.5
s.unregister(wr)
s.register(rd, selectors.EVENT_READ)
t = time()

assert not s.select(0)

assert not s.select(-1)

assert time() - t < 0.5
t0 = time()

assert not s.select(1)
t1 = time()
dt = t1 - t0

assert 0.8 <= dt <= 2.0
print("DefaultSelectorTestCase::test_timeout: ok")
