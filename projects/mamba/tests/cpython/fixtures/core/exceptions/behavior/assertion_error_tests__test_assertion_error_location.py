# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "core"
# lib = "exceptions"
# dimension = "behavior"
# case = "assertion_error_tests__test_assertion_error_location"
# subject = "cpython.test_exceptions.AssertionErrorTests.test_assertion_error_location"
# kind = "semantic"
# xfail = "auto-ported CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_exceptions.py"
# status = "filled"
# ///
# mamba-xfail: auto-ported CPython test; mamba promotion pending
# Auto-ported from CPython 3.12 test_exceptions.py::AssertionErrorTests::test_assertion_error_location
"""Auto-ported test: AssertionErrorTests::test_assertion_error_location (CPython 3.12 oracle)."""


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
cases = [('assert None', ['    assert None', '           ^^^^', 'AssertionError']), ('assert 0', ['    assert 0', '           ^', 'AssertionError']), ('assert 1 > 2', ['    assert 1 > 2', '           ^^^^^', 'AssertionError']), ('assert 1 > 2 and 3 > 2', ['    assert 1 > 2 and 3 > 2', '           ^^^^^^^^^^^^^^^', 'AssertionError']), ('assert 1 > 2, "messäge"', ['    assert 1 > 2, "messäge"', '           ^^^^^', 'AssertionError: messäge']), ('assert 1 > 2, "messäge"'.encode(), ['    assert 1 > 2, "messäge"', '           ^^^^^', 'AssertionError: messäge']), ('# coding: latin1\nassert 1 > 2, "messäge"'.encode('latin1'), ['    assert 1 > 2, "messäge"', '           ^^^^^', 'AssertionError: messäge']), ('\n             assert (\n                 1 > 2)\n             ', ['    1 > 2)', '    ^^^^^', 'AssertionError']), ('\n             assert (\n                 1 > 2), "Message"\n             ', ['    1 > 2), "Message"', '    ^^^^^', 'AssertionError: Message']), ('\n             assert (\n                 1 > 2), \\\n                 "Message"\n             ', ['    1 > 2), \\', '    ^^^^^', 'AssertionError: Message'])]
for source, expected in cases:
    result = run_script(source)

    assert result[-3:] == expected
print("AssertionErrorTests::test_assertion_error_location: ok")
