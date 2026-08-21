# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "events"
# dimension = "behavior"
# case = "policy_tests__test_new_event_loop"
# subject = "cpython.test_events.PolicyTests.test_new_event_loop"
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
loop = policy.new_event_loop()
assert isinstance(loop, asyncio.AbstractEventLoop)
loop.close()

print("PolicyTests::test_new_event_loop: ok")
