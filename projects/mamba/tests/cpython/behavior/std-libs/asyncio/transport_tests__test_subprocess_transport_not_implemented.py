# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "asyncio"
# dimension = "behavior"
# case = "transport_tests__test_subprocess_transport_not_implemented"
# subject = "cpython.test_transports.TransportTests.test_subprocess_transport_not_implemented"
# kind = "semantic"
# xfail = "auto-ported CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_asyncio/test_transports.py"
# status = "filled"
# ///
# mamba-xfail: auto-ported CPython test; mamba promotion pending
# Auto-ported from CPython 3.12 test_transports.py::TransportTests::test_subprocess_transport_not_implemented
"""Auto-ported test: TransportTests::test_subprocess_transport_not_implemented (CPython 3.12 oracle)."""


import unittest
from unittest import mock
import asyncio
from asyncio import transports


'Tests for transports.py.'

def tearDownModule():
    asyncio.set_event_loop_policy(None)


# --- test body ---
transport = asyncio.SubprocessTransport()

try:
    transport.get_pid()
    raise AssertionError('expected NotImplementedError')
except NotImplementedError:
    pass

try:
    transport.get_returncode()
    raise AssertionError('expected NotImplementedError')
except NotImplementedError:
    pass

try:
    transport.get_pipe_transport(1)
    raise AssertionError('expected NotImplementedError')
except NotImplementedError:
    pass

try:
    transport.send_signal(1)
    raise AssertionError('expected NotImplementedError')
except NotImplementedError:
    pass

try:
    transport.terminate()
    raise AssertionError('expected NotImplementedError')
except NotImplementedError:
    pass

try:
    transport.kill()
    raise AssertionError('expected NotImplementedError')
except NotImplementedError:
    pass
print("TransportTests::test_subprocess_transport_not_implemented: ok")
