# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "asyncio"
# dimension = "behavior"
# case = "c_task_future_tests__test_foobar"
# subject = "cpython.test_tasks.CTask_Future_Tests.test_foobar"
# kind = "semantic"
# xfail = "auto-ported CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_asyncio/test_tasks.py"
# status = "filled"
# ///
# mamba-xfail: auto-ported CPython test; mamba promotion pending
# Auto-ported from CPython 3.12 test_tasks.py::CTask_Future_Tests::test_foobar
"""Auto-ported test: CTask_Future_Tests::test_foobar (CPython 3.12 oracle)."""


import collections
import contextlib
import contextvars
import gc
import io
import random
import re
import sys
import traceback
import types
import unittest
from unittest import mock
from types import GenericAlias
import asyncio
from asyncio import futures
from asyncio import tasks
from test.test_asyncio import utils as test_utils
from test import support
from test.support.script_helper import assert_python_ok


'Tests for tasks.py.'

def tearDownModule():
    asyncio.set_event_loop_policy(None)

async def coroutine_function():
    pass

def format_coroutine(qualname, state, src, source_traceback, generator=False):
    if generator:
        state = '%s' % state
    else:
        state = '%s, defined' % state
    if source_traceback is not None:
        frame = source_traceback[-1]
        return 'coro=<%s() %s at %s> created at %s:%s' % (qualname, state, src, frame[0], frame[1])
    else:
        return 'coro=<%s() %s at %s>' % (qualname, state, src)

def get_innermost_context(exc):
    """
    Return information about the innermost exception context in the chain.
    """
    depth = 0
    while True:
        context = exc.__context__
        if context is None:
            break
        exc = context
        depth += 1
    return (type(exc), exc.args, depth)

class Dummy:

    def __repr__(self):
        return '<Dummy>'

    def __call__(self, *args):
        pass

class CoroLikeObject:

    def send(self, v):
        raise StopIteration(42)

    def throw(self, *exc):
        pass

    def close(self):
        pass

    def __await__(self):
        return self

def add_subclass_tests(cls):
    BaseTask = cls.Task
    BaseFuture = cls.Future
    if BaseTask is None or BaseFuture is None:
        return cls

    class CommonFuture:

        def __init__(self, *args, **kwargs):
            self.calls = collections.defaultdict(lambda: 0)
            super().__init__(*args, **kwargs)

        def add_done_callback(self, *args, **kwargs):
            self.calls['add_done_callback'] += 1
            return super().add_done_callback(*args, **kwargs)

    class Task(CommonFuture, BaseTask):
        pass

    class Future(CommonFuture, BaseFuture):
        pass

    def test_subclasses_ctask_cfuture(self):
        fut = self.Future(loop=self.loop)

        async def func():
            self.loop.call_soon(lambda: fut.set_result('spam'))
            return await fut
        task = self.Task(func(), loop=self.loop)
        result = self.loop.run_until_complete(task)
        self.assertEqual(result, 'spam')
        self.assertEqual(dict(task.calls), {'add_done_callback': 1})
        self.assertEqual(dict(fut.calls), {'add_done_callback': 1})
    cls.Task = Task
    cls.Future = Future
    cls.test_subclasses_ctask_cfuture = test_subclasses_ctask_cfuture
    cls.test_task_source_traceback = None
    return cls


# --- test body ---
class Fut(asyncio.Future):

    @property
    def get_loop(self):
        raise AttributeError

async def coro():
    await fut
    return 'spam'
self_loop = asyncio.new_event_loop()
try:
    fut = Fut(loop=self_loop)
    self_loop.call_later(0.1, fut.set_result, 1)
    task = self_loop.create_task(coro())
    res = self_loop.run_until_complete(task)
finally:
    self_loop.close()

assert res == 'spam'
print("CTask_Future_Tests::test_foobar: ok")
