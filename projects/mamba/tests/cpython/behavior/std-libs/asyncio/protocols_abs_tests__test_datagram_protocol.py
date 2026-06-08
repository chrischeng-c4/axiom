# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "asyncio"
# dimension = "behavior"
# case = "protocols_abs_tests__test_datagram_protocol"
# subject = "cpython.test_protocols.ProtocolsAbsTests.test_datagram_protocol"
# kind = "semantic"
# xfail = "auto-ported CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_asyncio/test_protocols.py"
# status = "filled"
# ///
# mamba-xfail: auto-ported CPython test; mamba promotion pending
# Auto-ported from CPython 3.12 test_protocols.py::ProtocolsAbsTests::test_datagram_protocol
"""Auto-ported test: ProtocolsAbsTests::test_datagram_protocol (CPython 3.12 oracle)."""


import unittest
from unittest import mock
import asyncio


def tearDownModule():
    asyncio.set_event_loop_policy(None)


# --- test body ---
f = mock.Mock()
dp = asyncio.DatagramProtocol()

assert dp.connection_made(f) is None

assert dp.connection_lost(f) is None

assert dp.error_received(f) is None

assert dp.datagram_received(f, f) is None

assert not hasattr(dp, '__dict__')
print("ProtocolsAbsTests::test_datagram_protocol: ok")
