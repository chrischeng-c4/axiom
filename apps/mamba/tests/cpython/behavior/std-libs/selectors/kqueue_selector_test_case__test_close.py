# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "selectors"
# dimension = "behavior"
# case = "kqueue_selector_test_case__test_close"
# subject = "cpython.test_selectors.KqueueSelectorTestCase.test_close"
# kind = "semantic"
# xfail = "auto-ported CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_selectors.py"
# status = "filled"
# ///
# mamba-xfail: auto-ported CPython test; mamba promotion pending
# Auto-ported from CPython 3.12 test_selectors.py::KqueueSelectorTestCase::test_close
"""Auto-ported test: KqueueSelectorTestCase::test_close (CPython 3.12 oracle)."""


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
SELECTOR = getattr(selectors, 'KqueueSelector', None)

def make_socketpair():
    rd, wr = socketpair()
    pass
    pass
    return (rd, wr)
s = SELECTOR()
pass
mapping = s.get_map()
rd, wr = make_socketpair()
s.register(rd, selectors.EVENT_READ)
s.register(wr, selectors.EVENT_WRITE)
s.close()

try:
    s.get_key(rd)
    raise AssertionError('expected RuntimeError')
except RuntimeError:
    pass

try:
    s.get_key(wr)
    raise AssertionError('expected RuntimeError')
except RuntimeError:
    pass

try:
    mapping.__getitem__(rd)
    raise AssertionError('expected KeyError')
except KeyError:
    pass

try:
    mapping.__getitem__(wr)
    raise AssertionError('expected KeyError')
except KeyError:
    pass
print("KqueueSelectorTestCase::test_close: ok")
