# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "timeout"
# dimension = "behavior"
# case = "creation_test_case__test_return_type"
# subject = "cpython.test_timeout.CreationTestCase.testReturnType"
# kind = "semantic"
# xfail = "auto-ported CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_timeout.py"
# status = "filled"
# ///
# mamba-xfail: auto-ported CPython test; mamba promotion pending
# Auto-ported from CPython 3.12 test_timeout.py::CreationTestCase::testReturnType
"""Auto-ported test: CreationTestCase::testReturnType (CPython 3.12 oracle)."""


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
self_sock.settimeout(1)

assert type(self_sock.gettimeout()) is float
self_sock.settimeout(3.9)

assert type(self_sock.gettimeout()) is float
print("CreationTestCase::testReturnType: ok")
