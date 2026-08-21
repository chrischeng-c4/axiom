# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "asyncio"
# dimension = "behavior"
# case = "transport_tests__test_get_extra_info"
# subject = "cpython.test_transports.TransportTests.test_get_extra_info"
# kind = "semantic"
# xfail = "auto-ported CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_asyncio/test_transports.py"
# status = "filled"
# ///
# mamba-xfail: auto-ported CPython test; mamba promotion pending
# Auto-ported from CPython 3.12 test_transports.py::TransportTests::test_get_extra_info
"""Auto-ported test: TransportTests::test_get_extra_info (CPython 3.12 oracle)."""


import unittest
from unittest import mock
import asyncio
from asyncio import transports


'Tests for transports.py.'

def tearDownModule():
    asyncio.set_event_loop_policy(None)


# --- test body ---
transport = asyncio.Transport({'extra': 'info'})

assert 'info' == transport.get_extra_info('extra')

assert transport.get_extra_info('unknown') is None
default = object()

assert default is transport.get_extra_info('unknown', default)
print("TransportTests::test_get_extra_info: ok")
