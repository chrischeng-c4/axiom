# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "selectors"
# dimension = "behavior"
# case = "select_selector_test_case__test_get_map"
# subject = "cpython.test_selectors.SelectSelectorTestCase.test_get_map"
# kind = "semantic"
# xfail = "auto-ported CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_selectors.py"
# status = "filled"
# ///
# mamba-xfail: auto-ported CPython test; mamba promotion pending
# Auto-ported from CPython 3.12 test_selectors.py::SelectSelectorTestCase::test_get_map
"""Auto-ported test: SelectSelectorTestCase::test_get_map (CPython 3.12 oracle)."""


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
SELECTOR = selectors.SelectSelector

def make_socketpair():
    rd, wr = socketpair()
    pass
    pass
    return (rd, wr)
s = SELECTOR()
pass
rd, wr = make_socketpair()
keys = s.get_map()

assert not keys

assert len(keys) == 0

assert list(keys) == []
key = s.register(rd, selectors.EVENT_READ, 'data')

assert rd in keys

assert key == keys[rd]

assert len(keys) == 1

assert list(keys) == [rd.fileno()]

assert list(keys.values()) == [key]
try:
    keys[999999]
    raise AssertionError('expected KeyError')
except KeyError:
    pass
try:
    del keys[rd]
    raise AssertionError('expected TypeError')
except TypeError:
    pass
print("SelectSelectorTestCase::test_get_map: ok")
