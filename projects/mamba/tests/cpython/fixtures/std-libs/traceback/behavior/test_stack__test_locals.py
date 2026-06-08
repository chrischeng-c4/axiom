# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "traceback"
# dimension = "behavior"
# case = "test_stack__test_locals"
# subject = "cpython.test_traceback.TestStack.test_locals"
# kind = "semantic"
# xfail = "auto-ported CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_traceback.py"
# status = "filled"
# ///
# mamba-xfail: auto-ported CPython test; mamba promotion pending
# Auto-ported from CPython 3.12 test_traceback.py::TestStack::test_locals
"""Auto-ported test: TestStack::test_locals (CPython 3.12 oracle)."""


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
linecache.updatecache('/foo.py', globals())
c = test_code('/foo.py', 'method')
f = test_frame(c, globals(), {'something': 1})
s = traceback.StackSummary.extract(iter([(f, 6)]), capture_locals=True)

assert s[0].locals == {'something': '1'}
print("TestStack::test_locals: ok")
