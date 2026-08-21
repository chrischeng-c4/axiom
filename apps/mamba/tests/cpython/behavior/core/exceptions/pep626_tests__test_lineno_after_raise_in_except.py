# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "core"
# lib = "exceptions"
# dimension = "behavior"
# case = "pep626_tests__test_lineno_after_raise_in_except"
# subject = "cpython.test_exceptions.PEP626Tests.test_lineno_after_raise_in_except"
# kind = "semantic"
# xfail = "auto-ported CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_exceptions.py"
# status = "filled"
# ///
# mamba-xfail: auto-ported CPython test; mamba promotion pending
# Auto-ported from CPython 3.12 test_exceptions.py::PEP626Tests::test_lineno_after_raise_in_except
"""Auto-ported test: PEP626Tests::test_lineno_after_raise_in_except (CPython 3.12 oracle)."""


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
def lineno_after_raise(f, *expected):
    try:
        f()
    except Exception as ex:
        t = ex.__traceback__
    else:

        raise AssertionError('No exception raised')
    lines = []
    t = t.tb_next
    while t:
        frame = t.tb_frame
        lines.append(None if frame.f_lineno is None else frame.f_lineno - frame.f_code.co_firstlineno)
        t = t.tb_next

    assert tuple(lines) == expected

def in_except():
    try:
        1 / 0
    except:
        1 / 0
        pass
lineno_after_raise(in_except, 4)
print("PEP626Tests::test_lineno_after_raise_in_except: ok")
