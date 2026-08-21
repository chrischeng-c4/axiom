# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "asyncio"
# dimension = "behavior"
# case = "protocols_abs_tests__test_protocol"
# subject = "cpython.test_protocols.ProtocolsAbsTests.test_protocol"
# kind = "semantic"
# xfail = "auto-ported CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_asyncio/test_protocols.py"
# status = "filled"
# ///
# mamba-xfail: auto-ported CPython test; mamba promotion pending
# Auto-ported from CPython 3.12 test_protocols.py::ProtocolsAbsTests::test_protocol
"""Auto-ported test: ProtocolsAbsTests::test_protocol (CPython 3.12 oracle)."""


import unittest
from unittest import mock
import asyncio


def tearDownModule():
    asyncio.set_event_loop_policy(None)


# --- test body ---
f = mock.Mock()
p = asyncio.Protocol()

assert p.connection_made(f) is None

assert p.connection_lost(f) is None

assert p.data_received(f) is None

assert p.eof_received() is None

assert p.pause_writing() is None

assert p.resume_writing() is None

assert not hasattr(p, '__dict__')
print("ProtocolsAbsTests::test_protocol: ok")
