# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "traceback"
# dimension = "behavior"
# case = "limit_tests__test_extract_tb"
# subject = "cpython.test_traceback.LimitTests.test_extract_tb"
# kind = "semantic"
# xfail = "auto-ported CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_traceback.py"
# status = "filled"
# ///
# mamba-xfail: auto-ported CPython test; mamba promotion pending
# Auto-ported from CPython 3.12 test_traceback.py::LimitTests::test_extract_tb
"""Auto-ported test: LimitTests::test_extract_tb (CPython 3.12 oracle)."""


from collections import namedtuple
from io import StringIO
import linecache
import sys
import types
import inspect
import builtins
import unittest
import re
import tempfile
import random
import string
from test import support
import shutil
from test.support import Error, captured_output, cpython_only, ALWAYS_EQ, requires_debug_ranges, has_no_debug_ranges, requires_subprocess
from test.support.os_helper import TESTFN, unlink
from test.support.script_helper import assert_python_ok, assert_python_failure
from test.support.import_helper import forget
import json
import textwrap
import traceback
from functools import partial
from pathlib import Path


'Test cases for traceback module'

MODULE_PREFIX = f'{__name__}.' if __name__ == '__main__' else ''

test_code = namedtuple('code', ['co_filename', 'co_name'])

test_code.co_positions = lambda _: iter([(6, 6, 0, 0)])

test_frame = namedtuple('frame', ['f_code', 'f_globals', 'f_locals'])

test_tb = namedtuple('tb', ['tb_frame', 'tb_lineno', 'tb_next', 'tb_lasti'])

LEVENSHTEIN_DATA_FILE = Path(__file__).parent / 'levenshtein_examples.json'

class PurePythonExceptionFormattingMixin:

    def get_exception(self, callable, slice_start=0, slice_end=-1):
        try:
            callable()
        except BaseException:
            return traceback.format_exc().splitlines()[slice_start:slice_end]
        else:
            self.fail('No exception thrown.')
    callable_line = get_exception.__code__.co_firstlineno + 2

class CAPIExceptionFormattingMixin:
    LEGACY = 0

    def get_exception(self, callable, slice_start=0, slice_end=-1):
        from _testcapi import exception_print
        try:
            callable()
            self.fail('No exception thrown.')
        except Exception as e:
            with captured_output('stderr') as tbstderr:
                exception_print(e, self.LEGACY)
            return tbstderr.getvalue().splitlines()[slice_start:slice_end]
    callable_line = get_exception.__code__.co_firstlineno + 3

class CAPIExceptionFormattingLegacyMixin(CAPIExceptionFormattingMixin):
    LEGACY = 1

cause_message = '\nThe above exception was the direct cause of the following exception:\n\n'

context_message = '\nDuring handling of the above exception, another exception occurred:\n\n'

boundaries = re.compile('(%s|%s)' % (re.escape(cause_message), re.escape(context_message)))

class Unrepresentable:

    def __repr__(self) -> str:
        raise Exception('Unrepresentable')

global_for_suggestions = None


# --- test body ---
def last_raises1():
    raise Exception('Last raised')

def last_raises2():
    last_raises1()

def last_raises3():
    last_raises2()

def last_raises4():
    last_raises3()

def last_raises5():
    last_raises4()

def last_returns_frame1():
    return sys._getframe()

def last_returns_frame2():
    return last_returns_frame1()

def last_returns_frame3():
    return last_returns_frame2()

def last_returns_frame4():
    return last_returns_frame3()

def last_returns_frame5():
    return last_returns_frame4()
try:
    last_raises5()
except Exception as e:
    tb = e.__traceback__

def extract(**kwargs):
    return traceback.extract_tb(tb, **kwargs)
with support.swap_attr(sys, 'tracebacklimit', 1000):
    nolim = extract()

    assert len(nolim) == 5 + 1

    assert extract(limit=2) == nolim[:2]

    assert extract(limit=10) == nolim

    assert extract(limit=-2) == nolim[-2:]

    assert extract(limit=-10) == nolim

    assert extract(limit=0) == []
    del sys.tracebacklimit

    assert extract() == nolim
    sys.tracebacklimit = 2

    assert extract() == nolim[:2]

    assert extract(limit=3) == nolim[:3]

    assert extract(limit=-3) == nolim[-3:]
    sys.tracebacklimit = 0

    assert extract() == []
    sys.tracebacklimit = -1

    assert extract() == []
print("LimitTests::test_extract_tb: ok")
