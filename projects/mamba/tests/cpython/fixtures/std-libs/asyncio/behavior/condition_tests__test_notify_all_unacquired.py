# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "asyncio"
# dimension = "behavior"
# case = "condition_tests__test_notify_all_unacquired"
# subject = "cpython.test_locks.ConditionTests.test_notify_all_unacquired"
# kind = "semantic"
# xfail = "auto-ported CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_asyncio/test_locks.py"
# status = "filled"
# ///
# mamba-xfail: auto-ported CPython test; mamba promotion pending
# Auto-ported from CPython 3.12 test_locks.py::ConditionTests::test_notify_all_unacquired
"""Auto-ported test: ConditionTests::test_notify_all_unacquired (CPython 3.12 oracle)."""


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
cond = asyncio.Condition()

try:
    cond.notify_all()
    raise AssertionError('expected RuntimeError')
except RuntimeError:
    pass
print("ConditionTests::test_notify_all_unacquired: ok")
