# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "contextlib"
# dimension = "behavior"
# case = "context_manager_test_case__test_contextmanager_attribs"
# subject = "cpython.test_contextlib.ContextManagerTestCase.test_contextmanager_attribs"
# kind = "semantic"
# xfail = "auto-ported CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_contextlib.py"
# status = "filled"
# ///
# mamba-xfail: auto-ported CPython test; mamba promotion pending
# Auto-ported from CPython 3.12 test_contextlib.py::ContextManagerTestCase::test_contextmanager_attribs
"""Auto-ported test: ContextManagerTestCase::test_contextmanager_attribs (CPython 3.12 oracle)."""


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
baz = _create_contextmanager_attribs()

assert baz.__name__ == 'baz'

assert baz.foo == 'bar'
print("ContextManagerTestCase::test_contextmanager_attribs: ok")
