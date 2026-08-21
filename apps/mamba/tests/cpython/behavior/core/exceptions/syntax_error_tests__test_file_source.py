# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "core"
# lib = "exceptions"
# dimension = "behavior"
# case = "syntax_error_tests__test_file_source"
# subject = "cpython.test_exceptions.SyntaxErrorTests.test_file_source"
# kind = "semantic"
# xfail = "auto-ported CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_exceptions.py"
# status = "filled"
# ///
# mamba-xfail: auto-ported CPython test; mamba promotion pending
# Auto-ported from CPython 3.12 test_exceptions.py::SyntaxErrorTests::test_file_source
"""Auto-ported test: SyntaxErrorTests::test_file_source (CPython 3.12 oracle)."""


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
pass
err = run_script('return "ä"')

assert err[-3::2] == ['    return "ä"', "SyntaxError: 'return' outside function"]
err = run_script('return "ä"'.encode())

assert err[-3::2] == ['    return "ä"', "SyntaxError: 'return' outside function"]
err = run_script(BOM_UTF8 + 'return "ä"'.encode())

assert err[-3::2] == ['    return "ä"', "SyntaxError: 'return' outside function"]
err = run_script('# coding: latin1\nreturn "ä"'.encode('latin1'))

assert err[-3::2] == ['    return "ä"', "SyntaxError: 'return' outside function"]
err = run_script('return "ä" #' + 'ä' * 1000)

assert err[-2:] == ['    ^^^^^^^^^^^', "SyntaxError: 'return' outside function"]

assert err[-3][:100] == '    return "ä" #' + 'ä' * 84
err = run_script('return "ä" # ' + 'ä' * 1000)

assert err[-2:] == ['    ^^^^^^^^^^^', "SyntaxError: 'return' outside function"]

assert err[-3][:100] == '    return "ä" # ' + 'ä' * 83
print("SyntaxErrorTests::test_file_source: ok")
