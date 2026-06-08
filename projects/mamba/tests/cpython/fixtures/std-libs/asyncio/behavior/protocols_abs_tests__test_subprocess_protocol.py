# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "asyncio"
# dimension = "behavior"
# case = "protocols_abs_tests__test_subprocess_protocol"
# subject = "cpython.test_protocols.ProtocolsAbsTests.test_subprocess_protocol"
# kind = "semantic"
# xfail = "auto-ported CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_asyncio/test_protocols.py"
# status = "filled"
# ///
# mamba-xfail: auto-ported CPython test; mamba promotion pending
# Auto-ported from CPython 3.12 test_protocols.py::ProtocolsAbsTests::test_subprocess_protocol
"""Auto-ported test: ProtocolsAbsTests::test_subprocess_protocol (CPython 3.12 oracle)."""


import unittest
from unittest import mock
import asyncio


def tearDownModule():
    asyncio.set_event_loop_policy(None)


# --- test body ---
f = mock.Mock()
sp = asyncio.SubprocessProtocol()

assert sp.connection_made(f) is None

assert sp.connection_lost(f) is None

assert sp.pipe_data_received(1, f) is None

assert sp.pipe_connection_lost(1, f) is None

assert sp.process_exited() is None

assert not hasattr(sp, '__dict__')
print("ProtocolsAbsTests::test_subprocess_protocol: ok")
