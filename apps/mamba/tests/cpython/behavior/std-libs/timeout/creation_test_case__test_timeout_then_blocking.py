# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "timeout"
# dimension = "behavior"
# case = "creation_test_case__test_timeout_then_blocking"
# subject = "cpython.test_timeout.CreationTestCase.testTimeoutThenBlocking"
# kind = "semantic"
# xfail = "auto-ported CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_timeout.py"
# status = "filled"
# ///
# mamba-xfail: auto-ported CPython test; mamba promotion pending
# Auto-ported from CPython 3.12 test_timeout.py::CreationTestCase::testTimeoutThenBlocking
"""Auto-ported test: CreationTestCase::testTimeoutThenBlocking (CPython 3.12 oracle)."""


import functools
import unittest
from test import support
from test.support import socket_helper
import time
import errno
import socket


'Unit tests for socket timeout feature.'

@functools.lru_cache()
def resolve_address(host, port):
    """Resolve an (host, port) to an address.

    We must perform name resolution before timeout tests, otherwise it will be
    performed by connect().
    """
    with socket_helper.transient_internet(host):
        return socket.getaddrinfo(host, port, socket.AF_INET, socket.SOCK_STREAM)[0][4]

def setUpModule():
    support.requires('network')
    support.requires_working_socket(module=True)


# --- test body ---
self_sock = socket.socket(socket.AF_INET, socket.SOCK_STREAM)
self_sock.settimeout(10)
self_sock.setblocking(True)

assert self_sock.gettimeout() == None
self_sock.setblocking(False)

assert self_sock.gettimeout() == 0.0
self_sock.settimeout(10)
self_sock.setblocking(False)

assert self_sock.gettimeout() == 0.0
self_sock.setblocking(True)

assert self_sock.gettimeout() == None
print("CreationTestCase::testTimeoutThenBlocking: ok")
