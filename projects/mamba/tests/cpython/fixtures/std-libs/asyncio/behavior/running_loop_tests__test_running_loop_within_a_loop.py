# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "asyncio"
# dimension = "behavior"
# case = "running_loop_tests__test_running_loop_within_a_loop"
# subject = "cpython.test_base_events.RunningLoopTests.test_running_loop_within_a_loop"
# kind = "semantic"
# xfail = "auto-ported CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_asyncio/test_base_events.py"
# status = "filled"
# ///
# mamba-xfail: auto-ported CPython test; mamba promotion pending
# Auto-ported from CPython 3.12 test_base_events.py::RunningLoopTests::test_running_loop_within_a_loop
"""Auto-ported test: RunningLoopTests::test_running_loop_within_a_loop (CPython 3.12 oracle)."""


import concurrent.futures
import errno
import math
import socket
import sys
import threading
import time
import unittest
from unittest import mock
import asyncio
from asyncio import base_events
from asyncio import constants
from test.test_asyncio import utils as test_utils
from test import support
from test.support.script_helper import assert_python_ok
from test.support import os_helper
from test.support import socket_helper
import warnings


'Tests for base_events.py'

MOCK_ANY = mock.ANY

def tearDownModule():
    asyncio.set_event_loop_policy(None)

def mock_socket_module():
    m_socket = mock.MagicMock(spec=socket)
    for name in ('AF_INET', 'AF_INET6', 'AF_UNSPEC', 'IPPROTO_TCP', 'IPPROTO_UDP', 'SOCK_STREAM', 'SOCK_DGRAM', 'SOL_SOCKET', 'SO_REUSEADDR', 'inet_pton'):
        if hasattr(socket, name):
            setattr(m_socket, name, getattr(socket, name))
        else:
            delattr(m_socket, name)
    m_socket.socket = mock.MagicMock()
    m_socket.socket.return_value = test_utils.mock_nonblocking_socket()
    return m_socket

def patch_socket(f):
    return mock.patch('asyncio.base_events.socket', new_callable=mock_socket_module)(f)

class MyProto(asyncio.Protocol):
    done = None

    def __init__(self, create_future=False):
        self.state = 'INITIAL'
        self.nbytes = 0
        if create_future:
            self.done = asyncio.get_running_loop().create_future()

    def _assert_state(self, *expected):
        if self.state not in expected:
            raise AssertionError(f'state: {self.state!r}, expected: {expected!r}')

    def connection_made(self, transport):
        self.transport = transport
        self._assert_state('INITIAL')
        self.state = 'CONNECTED'
        transport.write(b'GET / HTTP/1.0\r\nHost: example.com\r\n\r\n')

    def data_received(self, data):
        self._assert_state('CONNECTED')
        self.nbytes += len(data)

    def eof_received(self):
        self._assert_state('CONNECTED')
        self.state = 'EOF'

    def connection_lost(self, exc):
        self._assert_state('CONNECTED', 'EOF')
        self.state = 'CLOSED'
        if self.done:
            self.done.set_result(None)

class MyDatagramProto(asyncio.DatagramProtocol):
    done = None

    def __init__(self, create_future=False, loop=None):
        self.state = 'INITIAL'
        self.nbytes = 0
        if create_future:
            self.done = loop.create_future()

    def _assert_state(self, expected):
        if self.state != expected:
            raise AssertionError(f'state: {self.state!r}, expected: {expected!r}')

    def connection_made(self, transport):
        self.transport = transport
        self._assert_state('INITIAL')
        self.state = 'INITIALIZED'

    def datagram_received(self, data, addr):
        self._assert_state('INITIALIZED')
        self.nbytes += len(data)

    def error_received(self, exc):
        self._assert_state('INITIALIZED')

    def connection_lost(self, exc):
        self._assert_state('INITIALIZED')
        self.state = 'CLOSED'
        if self.done:
            self.done.set_result(None)


# --- test body ---
async def runner(loop):
    loop.run_forever()
loop = asyncio.new_event_loop()
outer_loop = asyncio.new_event_loop()
try:
    try:
        outer_loop.run_until_complete(runner(loop))
        raise AssertionError('expected RuntimeError')
    except RuntimeError as _aR_e:
        import re as _re_aR
        assert _re_aR.search('while another loop is running', str(_aR_e))
finally:
    loop.close()
    outer_loop.close()
print("RunningLoopTests::test_running_loop_within_a_loop: ok")
