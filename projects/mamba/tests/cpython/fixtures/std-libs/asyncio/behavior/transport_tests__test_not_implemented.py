# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "asyncio"
# dimension = "behavior"
# case = "transport_tests__test_not_implemented"
# subject = "cpython.test_transports.TransportTests.test_not_implemented"
# kind = "semantic"
# xfail = "auto-ported CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_asyncio/test_transports.py"
# status = "filled"
# ///
# mamba-xfail: auto-ported CPython test; mamba promotion pending
# Auto-ported from CPython 3.12 test_transports.py::TransportTests::test_not_implemented
"""Auto-ported test: TransportTests::test_not_implemented (CPython 3.12 oracle)."""


import unittest
from unittest import mock
import asyncio
from asyncio import transports


'Tests for transports.py.'

def tearDownModule():
    asyncio.set_event_loop_policy(None)


# --- test body ---
transport = asyncio.Transport()

try:
    transport.set_write_buffer_limits()
    raise AssertionError('expected NotImplementedError')
except NotImplementedError:
    pass

try:
    transport.get_write_buffer_size()
    raise AssertionError('expected NotImplementedError')
except NotImplementedError:
    pass

try:
    transport.write('data')
    raise AssertionError('expected NotImplementedError')
except NotImplementedError:
    pass

try:
    transport.write_eof()
    raise AssertionError('expected NotImplementedError')
except NotImplementedError:
    pass

try:
    transport.can_write_eof()
    raise AssertionError('expected NotImplementedError')
except NotImplementedError:
    pass

try:
    transport.pause_reading()
    raise AssertionError('expected NotImplementedError')
except NotImplementedError:
    pass

try:
    transport.resume_reading()
    raise AssertionError('expected NotImplementedError')
except NotImplementedError:
    pass

try:
    transport.is_reading()
    raise AssertionError('expected NotImplementedError')
except NotImplementedError:
    pass

try:
    transport.close()
    raise AssertionError('expected NotImplementedError')
except NotImplementedError:
    pass

try:
    transport.abort()
    raise AssertionError('expected NotImplementedError')
except NotImplementedError:
    pass
print("TransportTests::test_not_implemented: ok")
