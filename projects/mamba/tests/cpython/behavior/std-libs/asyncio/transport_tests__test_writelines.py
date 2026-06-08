# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "asyncio"
# dimension = "behavior"
# case = "transport_tests__test_writelines"
# subject = "cpython.test_transports.TransportTests.test_writelines"
# kind = "semantic"
# xfail = "auto-ported CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_asyncio/test_transports.py"
# status = "filled"
# ///
# mamba-xfail: auto-ported CPython test; mamba promotion pending
# Auto-ported from CPython 3.12 test_transports.py::TransportTests::test_writelines
"""Auto-ported test: TransportTests::test_writelines (CPython 3.12 oracle)."""


import unittest
from unittest import mock
import asyncio
from asyncio import transports


'Tests for transports.py.'

def tearDownModule():
    asyncio.set_event_loop_policy(None)


# --- test body ---
writer = mock.Mock()

class MyTransport(asyncio.Transport):

    def write(self, data):
        writer(data)
transport = MyTransport()
transport.writelines([b'line1', bytearray(b'line2'), memoryview(b'line3')])

assert 1 == writer.call_count
writer.assert_called_with(b'line1line2line3')
print("TransportTests::test_writelines: ok")
