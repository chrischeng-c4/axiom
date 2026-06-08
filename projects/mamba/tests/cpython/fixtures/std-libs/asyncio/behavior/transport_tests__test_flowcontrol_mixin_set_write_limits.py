# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "asyncio"
# dimension = "behavior"
# case = "transport_tests__test_flowcontrol_mixin_set_write_limits"
# subject = "cpython.test_transports.TransportTests.test_flowcontrol_mixin_set_write_limits"
# kind = "semantic"
# xfail = "auto-ported CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_asyncio/test_transports.py"
# status = "filled"
# ///
# mamba-xfail: auto-ported CPython test; mamba promotion pending
# Auto-ported from CPython 3.12 test_transports.py::TransportTests::test_flowcontrol_mixin_set_write_limits
"""Auto-ported test: TransportTests::test_flowcontrol_mixin_set_write_limits (CPython 3.12 oracle)."""


import unittest
from unittest import mock
import asyncio
from asyncio import transports


'Tests for transports.py.'

def tearDownModule():
    asyncio.set_event_loop_policy(None)


# --- test body ---
class MyTransport(transports._FlowControlMixin, transports.Transport):

    def get_write_buffer_size(self):
        return 512
loop = mock.Mock()
transport = MyTransport(loop=loop)
transport._protocol = mock.Mock()

assert not transport._protocol_paused
try:
    transport.set_write_buffer_limits(high=0, low=1)
    raise AssertionError('expected ValueError')
except ValueError as _aR_e:
    import re as _re_aR
    assert _re_aR.search('high.*must be >= low', str(_aR_e))
transport.set_write_buffer_limits(high=1024, low=128)

assert not transport._protocol_paused

assert transport.get_write_buffer_limits() == (128, 1024)
transport.set_write_buffer_limits(high=256, low=128)

assert transport._protocol_paused

assert transport.get_write_buffer_limits() == (128, 256)
print("TransportTests::test_flowcontrol_mixin_set_write_limits: ok")
