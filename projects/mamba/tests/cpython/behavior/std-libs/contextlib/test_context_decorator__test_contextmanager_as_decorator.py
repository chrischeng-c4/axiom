# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "contextlib"
# dimension = "behavior"
# case = "test_context_decorator__test_contextmanager_as_decorator"
# subject = "cpython.test_contextlib.TestContextDecorator.test_contextmanager_as_decorator"
# kind = "semantic"
# xfail = "auto-extracted CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_contextlib.py"
# status = "filled"
# ///
# mamba-xfail: auto-extracted CPython test; mamba promotion pending
import io
import os
import sys
import tempfile
import threading
import traceback
from contextlib import *
import weakref

@contextmanager
def woohoo(y):
    state.append(y)
    yield
    state.append(999)
state = []

@woohoo(1)
def test(x):
    assert state == [1]
    state.append(x)
test('something')
assert state == [1, 'something', 999]
state = []
test('something else')
assert state == [1, 'something else', 999]

print("TestContextDecorator::test_contextmanager_as_decorator: ok")
