# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "core"
# lib = "exceptions"
# dimension = "behavior"
# case = "exception_tests__test_recursion_normalizing_exception"
# subject = "cpython.test_exceptions.ExceptionTests.test_recursion_normalizing_exception"
# kind = "semantic"
# xfail = "auto-ported CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_exceptions.py"
# status = "filled"
# ///
# mamba-xfail: auto-ported CPython test; mamba promotion pending
# Auto-ported from CPython 3.12 test_exceptions.py::ExceptionTests::test_recursion_normalizing_exception
"""Auto-ported test: ExceptionTests::test_recursion_normalizing_exception (CPython 3.12 oracle)."""


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
code = "if 1:\n            import sys\n            from _testinternalcapi import get_recursion_depth\n            from test import support\n\n            class MyException(Exception): pass\n\n            def setrecursionlimit(depth):\n                while 1:\n                    try:\n                        sys.setrecursionlimit(depth)\n                        return depth\n                    except RecursionError:\n                        # sys.setrecursionlimit() raises a RecursionError if\n                        # the new recursion limit is too low (issue #25274).\n                        depth += 1\n\n            def recurse(cnt):\n                cnt -= 1\n                if cnt:\n                    recurse(cnt)\n                else:\n                    generator.throw(MyException)\n\n            def gen():\n                f = open(%a, mode='rb', buffering=0)\n                yield\n\n            generator = gen()\n            next(generator)\n            recursionlimit = sys.getrecursionlimit()\n            try:\n                recurse(support.EXCEEDS_RECURSION_LIMIT)\n            finally:\n                sys.setrecursionlimit(recursionlimit)\n                print('Done.')\n        " % __file__
rc, out, err = script_helper.assert_python_failure('-Wd', '-c', code)

assert rc == 1

assert b'RecursionError' in err

assert b'ResourceWarning' in err

assert b'Done.' in out
print("ExceptionTests::test_recursion_normalizing_exception: ok")
