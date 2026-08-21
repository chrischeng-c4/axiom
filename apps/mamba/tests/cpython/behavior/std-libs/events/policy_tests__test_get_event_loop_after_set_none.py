# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "events"
# dimension = "behavior"
# case = "policy_tests__test_get_event_loop_after_set_none"
# subject = "cpython.test_events.PolicyTests.test_get_event_loop_after_set_none"
# kind = "semantic"
# xfail = "auto-extracted CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_asyncio/test_events.py"
# status = "filled"
# ///
# mamba-xfail: auto-extracted CPython test; mamba promotion pending
import collections.abc
import concurrent.futures
import functools
import io
import multiprocessing
import os
import platform
import re
import signal
import socket
import subprocess
import sys
import threading
import time
import types
import errno
import weakref
import warnings
import asyncio
from asyncio import coroutines
from asyncio import events
from asyncio import selector_events
from multiprocessing.util import _cleanup_tests as multiprocessing_cleanup_tests
policy = asyncio.DefaultEventLoopPolicy()
policy.set_event_loop(None)
try:
    policy.get_event_loop()
    raise AssertionError('assertRaises: no raise')
except RuntimeError:
    pass

print("PolicyTests::test_get_event_loop_after_set_none: ok")
