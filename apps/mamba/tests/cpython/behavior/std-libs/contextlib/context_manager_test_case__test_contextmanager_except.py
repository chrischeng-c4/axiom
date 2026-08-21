# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "contextlib"
# dimension = "behavior"
# case = "context_manager_test_case__test_contextmanager_except"
# subject = "cpython.test_contextlib.ContextManagerTestCase.test_contextmanager_except"
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

def _create_contextmanager_attribs():

    def attribs(**kw):

        def decorate(func):
            for k, v in kw.items():
                setattr(func, k, v)
            return func
        return decorate

    @contextmanager
    @attribs(foo='bar')
    def baz(spam):
        """Whee!"""
        yield
    return baz
state = []

@contextmanager
def woohoo():
    state.append(1)
    try:
        yield 42
    except ZeroDivisionError as e:
        state.append(e.args[0])
        assert state == [1, 42, 999]
with woohoo() as x:
    assert state == [1]
    assert x == 42
    state.append(x)
    raise ZeroDivisionError(999)
assert state == [1, 42, 999]

print("ContextManagerTestCase::test_contextmanager_except: ok")
