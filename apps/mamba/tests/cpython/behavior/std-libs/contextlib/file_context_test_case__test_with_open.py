# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "contextlib"
# dimension = "behavior"
# case = "file_context_test_case__test_with_open"
# subject = "cpython.test_contextlib.FileContextTestCase.testWithOpen"
# kind = "semantic"
# xfail = "auto-ported CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_contextlib.py"
# status = "filled"
# ///
# mamba-xfail: auto-ported CPython test; mamba promotion pending
# Auto-ported from CPython 3.12 test_contextlib.py::FileContextTestCase::testWithOpen
"""Auto-ported test: FileContextTestCase::testWithOpen (CPython 3.12 oracle)."""


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
tfn = tempfile.mktemp()
try:
    with open(tfn, 'w', encoding='utf-8') as f:

        assert not f.closed
        f.write('Booh\n')

    assert f.closed
    try:
        with open(tfn, 'r', encoding='utf-8') as f:

            assert not f.closed

            assert f.read() == 'Booh\n'
            1 / 0
        raise AssertionError('expected ZeroDivisionError')
    except ZeroDivisionError:
        pass

    assert f.closed
finally:
    os_helper.unlink(tfn)
print("FileContextTestCase::testWithOpen: ok")
