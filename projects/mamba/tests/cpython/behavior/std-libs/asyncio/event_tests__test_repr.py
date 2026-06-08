# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "asyncio"
# dimension = "behavior"
# case = "event_tests__test_repr"
# subject = "cpython.test_locks.EventTests.test_repr"
# kind = "semantic"
# xfail = "auto-ported CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_asyncio/test_locks.py"
# status = "filled"
# ///
# mamba-xfail: auto-ported CPython test; mamba promotion pending
# Auto-ported from CPython 3.12 test_locks.py::EventTests::test_repr
"""Auto-ported test: EventTests::test_repr (CPython 3.12 oracle)."""


import unittest
from unittest import mock
import re
import asyncio
import collections


'Tests for locks.py'

STR_RGX_REPR = '^<(?P<class>.*?) object at (?P<address>.*?)\\[(?P<extras>(set|unset|locked|unlocked|filling|draining|resetting|broken)(, value:\\d)?(, waiters:\\d+)?(, waiters:\\d+\\/\\d+)?)\\]>\\Z'

RGX_REPR = re.compile(STR_RGX_REPR)

def tearDownModule():
    asyncio.set_event_loop_policy(None)


# --- test body ---
ev = asyncio.Event()

assert repr(ev).endswith('[unset]>')
match = RGX_REPR.match(repr(ev))

assert match.group('extras') == 'unset'
ev.set()

assert repr(ev).endswith('[set]>')

assert RGX_REPR.match(repr(ev))
ev._waiters.append(mock.Mock())

assert 'waiters:1' in repr(ev)

assert RGX_REPR.match(repr(ev))
print("EventTests::test_repr: ok")
