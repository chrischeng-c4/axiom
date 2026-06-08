# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "selectors"
# dimension = "behavior"
# case = "kqueue_selector_test_case__test_fileno"
# subject = "cpython.test_selectors.KqueueSelectorTestCase.test_fileno"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_selectors.py"
# status = "filled"
# ///
# Auto-ported from CPython 3.12 test_selectors.py::KqueueSelectorTestCase::test_fileno
"""Auto-ported test: KqueueSelectorTestCase::test_fileno (CPython 3.12 oracle)."""


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
if hasattr(s, 'fileno'):
    fd = s.fileno()

    assert isinstance(fd, int)

    assert fd >= 0
print("KqueueSelectorTestCase::test_fileno: ok")
