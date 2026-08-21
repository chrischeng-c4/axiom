# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "asyncio"
# dimension = "behavior"
# case = "transport_tests__test_ctor_extra_is_none"
# subject = "cpython.test_transports.TransportTests.test_ctor_extra_is_none"
# kind = "semantic"
# xfail = "auto-ported CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_asyncio/test_transports.py"
# status = "filled"
# ///
# mamba-xfail: auto-ported CPython test; mamba promotion pending
# Auto-ported from CPython 3.12 test_transports.py::TransportTests::test_ctor_extra_is_none
"""Auto-ported test: TransportTests::test_ctor_extra_is_none (CPython 3.12 oracle)."""


import unittest
from unittest import mock
import asyncio
from asyncio import transports


'Tests for transports.py.'

def tearDownModule():
    asyncio.set_event_loop_policy(None)


# --- test body ---
transport = asyncio.Transport()

assert transport._extra == {}
print("TransportTests::test_ctor_extra_is_none: ok")
