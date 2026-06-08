# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "traceback"
# dimension = "behavior"
# case = "traceback_cases__test_print_traceback_at_exit"
# subject = "cpython.test_traceback.TracebackCases.test_print_traceback_at_exit"
# kind = "semantic"
# xfail = "auto-ported CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_traceback.py"
# status = "filled"
# ///
# mamba-xfail: auto-ported CPython test; mamba promotion pending
# Auto-ported from CPython 3.12 test_traceback.py::TracebackCases::test_print_traceback_at_exit
"""Auto-ported test: TracebackCases::test_print_traceback_at_exit (CPython 3.12 oracle)."""


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
def get_exception_format(func, exc):
    try:
        func()
    except exc as value:
        return traceback.format_exception_only(exc, value)
    else:
        raise ValueError('call did not raise exception')

def syntax_error_bad_indentation():
    compile('def spam():\n  print(1)\n print(2)', '?', 'exec')

def syntax_error_bad_indentation2():
    compile(' print(2)', '?', 'exec')

def syntax_error_with_caret():
    compile('def fact(x):\n\treturn x!\n', '?', 'exec')

def syntax_error_with_caret_2():
    compile('1 +\n', '?', 'exec')

def syntax_error_with_caret_non_ascii():
    compile('Python = "Ṕýţĥòñ" +', '?', 'exec')

def syntax_error_with_caret_range():
    compile('f(x, y for y in range(30), z)', '?', 'exec')

def tokenizer_error_with_caret_range():
    compile('blech  (  ', '?', 'exec')
code = textwrap.dedent('\n            import sys\n            import traceback\n\n            class PrintExceptionAtExit(object):\n                def __init__(self):\n                    try:\n                        x = 1 / 0\n                    except Exception as e:\n                        self.exc = e\n                        # self.exc.__traceback__ contains frames:\n                        # explicitly clear the reference to self in the current\n                        # frame to break a reference cycle\n                        self = None\n\n                def __del__(self):\n                    traceback.print_exception(self.exc)\n\n            # Keep a reference in the module namespace to call the destructor\n            # when the module is unloaded\n            obj = PrintExceptionAtExit()\n        ')
rc, stdout, stderr = assert_python_ok('-c', code)
expected = [b'Traceback (most recent call last):', b'  File "<string>", line 8, in __init__', b'ZeroDivisionError: division by zero']

assert stderr.splitlines() == expected
print("TracebackCases::test_print_traceback_at_exit: ok")
