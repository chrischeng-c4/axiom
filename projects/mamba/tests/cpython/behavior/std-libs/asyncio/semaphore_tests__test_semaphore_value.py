# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "asyncio"
# dimension = "behavior"
# case = "semaphore_tests__test_semaphore_value"
# subject = "cpython.test_locks.SemaphoreTests.test_semaphore_value"
# kind = "semantic"
# xfail = "auto-ported CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_asyncio/test_locks.py"
# status = "filled"
# ///
# mamba-xfail: auto-ported CPython test; mamba promotion pending
# Auto-ported from CPython 3.12 test_locks.py::SemaphoreTests::test_semaphore_value
"""Auto-ported test: SemaphoreTests::test_semaphore_value (CPython 3.12 oracle)."""


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

try:
    asyncio.Semaphore(-1)
    raise AssertionError('expected ValueError')
except ValueError:
    pass
print("SemaphoreTests::test_semaphore_value: ok")
