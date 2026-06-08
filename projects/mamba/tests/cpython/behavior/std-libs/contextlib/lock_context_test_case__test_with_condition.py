# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "contextlib"
# dimension = "behavior"
# case = "lock_context_test_case__test_with_condition"
# subject = "cpython.test_contextlib.LockContextTestCase.testWithCondition"
# kind = "semantic"
# xfail = "auto-ported CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_contextlib.py"
# status = "filled"
# ///
# mamba-xfail: auto-ported CPython test; mamba promotion pending
# Auto-ported from CPython 3.12 test_contextlib.py::LockContextTestCase::testWithCondition
"""Auto-ported test: LockContextTestCase::testWithCondition (CPython 3.12 oracle)."""


import io
import os
import sys
import tempfile
import threading
import traceback
import unittest
from contextlib import *
from test import support
from test.support import os_helper
from test.support.testcase import ExceptionIsLikeMixin
import weakref


'Unit tests for contextlib.py, and other context managers.'

class mycontext(ContextDecorator):
    """Example decoration-compatible context manager for testing"""
    started = False
    exc = None
    catch = False

    def __enter__(self):
        self.started = True
        return self

    def __exit__(self, *exc):
        self.exc = exc
        return self.catch


# --- test body ---
def boilerPlate(lock, locked):

    assert not locked()
    with lock:

        assert locked()

    assert not locked()
    try:
        with lock:

            assert locked()
            1 / 0
        raise AssertionError('expected ZeroDivisionError')
    except ZeroDivisionError:
        pass

    assert not locked()
lock = threading.Condition()

def locked():
    return lock._is_owned()
boilerPlate(lock, locked)
print("LockContextTestCase::testWithCondition: ok")
