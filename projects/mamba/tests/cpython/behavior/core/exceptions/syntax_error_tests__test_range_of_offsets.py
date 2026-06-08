# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "core"
# lib = "exceptions"
# dimension = "behavior"
# case = "syntax_error_tests__test_range_of_offsets"
# subject = "cpython.test_exceptions.SyntaxErrorTests.test_range_of_offsets"
# kind = "semantic"
# xfail = "auto-ported CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_exceptions.py"
# status = "filled"
# ///
# mamba-xfail: auto-ported CPython test; mamba promotion pending
# Auto-ported from CPython 3.12 test_exceptions.py::SyntaxErrorTests::test_range_of_offsets
"""Auto-ported test: SyntaxErrorTests::test_range_of_offsets (CPython 3.12 oracle)."""


import copy
import os
import sys
import unittest
import pickle
import weakref
import errno
from codecs import BOM_UTF8
from itertools import product
from textwrap import dedent
from test.support import captured_stderr, check_impl_detail, cpython_only, gc_collect, no_tracing, script_helper, SuppressCrashReport
from test.support.import_helper import import_module
from test.support.os_helper import TESTFN, unlink
from test.support.warnings_helper import check_warnings
from test import support


try:
    from _testcapi import INT_MAX
except ImportError:
    INT_MAX = 2 ** 31 - 1

class NaiveException(Exception):

    def __init__(self, x):
        self.x = x

class SlottedNaiveException(Exception):
    __slots__ = ('x',)

    def __init__(self, x):
        self.x = x

class BrokenStrException(Exception):

    def __str__(self):
        raise Exception('str() is broken')

def run_script(source):
    if isinstance(source, str):
        with open(TESTFN, 'w', encoding='utf-8') as testfile:
            testfile.write(dedent(source))
    else:
        with open(TESTFN, 'wb') as testfile:
            testfile.write(source)
    _rc, _out, err = script_helper.assert_python_failure('-Wd', '-X', 'utf8', TESTFN)
    return err.decode('utf-8').splitlines()


# --- test body ---
maxDiff = None
cases = [(('bad.py', 1, 2, 'abcdefg', 1, 7), dedent('\n               File "bad.py", line 1\n                 abcdefg\n                  ^^^^^\n             SyntaxError: bad bad\n             ')), (('bad.py', 1, 2, 'abcdefg', 1, 3), dedent('\n               File "bad.py", line 1\n                 abcdefg\n                  ^\n             SyntaxError: bad bad\n             ')), (('bad.py', 1, 2, 'abcdefg', 1, -2), dedent('\n               File "bad.py", line 1\n                 abcdefg\n                  ^\n             SyntaxError: bad bad\n             ')), (('bad.py', 1, 4, 'abcdefg', 1, 2), dedent('\n               File "bad.py", line 1\n                 abcdefg\n                    ^\n             SyntaxError: bad bad\n             ')), (('bad.py', 1, -4, 'abcdefg', 1, -2), dedent('\n               File "bad.py", line 1\n                 abcdefg\n             SyntaxError: bad bad\n             ')), (('bad.py', 1, -4, 'abcdefg', 1, -5), dedent('\n               File "bad.py", line 1\n                 abcdefg\n             SyntaxError: bad bad\n             ')), (('bad.py', 1, 0, 'abcdefg', 1, 0), dedent('\n               File "bad.py", line 1\n                 abcdefg\n             SyntaxError: bad bad\n             ')), (('bad.py', 1, 0, 'abcdefg', 1, 5), dedent('\n               File "bad.py", line 1\n                 abcdefg\n             SyntaxError: bad bad\n             ')), (('bad.py', 1, 2, 'abcdefg', 1, 100), dedent('\n               File "bad.py", line 1\n                 abcdefg\n                  ^^^^^^\n             SyntaxError: bad bad\n             '))]
for args, expected in cases:
    try:
        raise SyntaxError('bad bad', args)
    except SyntaxError as exc:
        with support.captured_stderr() as err:
            sys.__excepthook__(*sys.exc_info())

        assert expected in err.getvalue()
        the_exception = exc
print("SyntaxErrorTests::test_range_of_offsets: ok")
