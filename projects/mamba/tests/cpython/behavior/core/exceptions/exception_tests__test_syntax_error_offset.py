# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "core"
# lib = "exceptions"
# dimension = "behavior"
# case = "exception_tests__test_syntax_error_offset"
# subject = "cpython.test_exceptions.ExceptionTests.testSyntaxErrorOffset"
# kind = "semantic"
# xfail = "auto-ported CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_exceptions.py"
# status = "filled"
# ///
# mamba-xfail: auto-ported CPython test; mamba promotion pending
# Auto-ported from CPython 3.12 test_exceptions.py::ExceptionTests::testSyntaxErrorOffset
"""Auto-ported test: ExceptionTests::testSyntaxErrorOffset (CPython 3.12 oracle)."""


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
def _check_generator_cleanup_exc_state(testfunc):

    class MyException(Exception):

        def __init__(self, obj):
            self.obj = obj

    class MyObj:
        pass

    def raising_gen():
        try:
            raise MyException(obj)
        except MyException:
            yield
    obj = MyObj()
    wr = weakref.ref(obj)
    g = raising_gen()
    next(g)
    testfunc(g)
    g = obj = None
    gc_collect()
    obj = wr()

    assert obj is None

def check(src, lineno, offset, end_lineno=None, end_offset=None, encoding='utf-8'):
    try:
        compile(src, '<fragment>', 'exec')
        raise AssertionError('expected SyntaxError')
    except SyntaxError as _aR_e:
        import types as _types_aR
        cm = _types_aR.SimpleNamespace(exception=_aR_e)

    assert cm.exception.lineno == lineno

    assert cm.exception.offset == offset
    if end_lineno is not None:

        assert cm.exception.end_lineno == end_lineno
    if end_offset is not None:

        assert cm.exception.end_offset == end_offset
    if cm.exception.text is not None:
        if not isinstance(src, str):
            src = src.decode(encoding, 'replace')
        line = src.split('\n')[lineno - 1]

        assert line in cm.exception.text

def raise_catch(exc, excname):
    try:
        raise exc('spam')
    except exc as err:
        buf1 = str(err)
    try:
        raise exc('spam')
    except exc as err:
        buf2 = str(err)

    assert buf1 == buf2

    assert exc.__name__ == excname
check = check
check('def fact(x):\n\treturn x!\n', 2, 10)
check('1 +\n', 1, 4)
check('def spam():\n  print(1)\n print(2)', 3, 10)
check('Python = "Python" +', 1, 20)
check('Python = "Ṕýţĥòñ" +', 1, 20)
check(b'# -*- coding: cp1251 -*-\nPython = "\xcf\xb3\xf2\xee\xed" +', 2, 19, encoding='cp1251')
check(b'Python = "\xcf\xb3\xf2\xee\xed" +', 1, 10)
check('x = "a', 1, 5)
check('lambda x: x = 2', 1, 1)
check('f{a + b + c}', 1, 2)
check('[file for str(file) in []\n]', 1, 11)
check('a = « hello » « world »', 1, 5)
check('[\nfile\nfor str(file)\nin\n[]\n]', 3, 5)
check('[file for\n str(file) in []]', 2, 2)
check("ages = {'Alice'=22, 'Bob'=23}", 1, 9)
check('match ...:\n    case {**rest, "key": value}:\n        ...', 2, 19)
check('[a b c d e f]', 1, 2)
check('for x yfff:', 1, 7)
check('f(a for a in b, c)', 1, 3, 1, 15)
check('f(a for a in b if a, c)', 1, 3, 1, 20)
check('f(a, b for b in c)', 1, 6, 1, 18)
check('f(a, b for b in c, d)', 1, 6, 1, 18)
check('class foo:return 1', 1, 11)
check('def f():\n  continue', 2, 3)
check('def f():\n  break', 2, 3)
check('try:\n  pass\nexcept:\n  pass\nexcept ValueError:\n  pass', 3, 1)
check('try:\n  pass\nexcept*:\n  pass', 3, 8)
check('try:\n  pass\nexcept*:\n  pass\nexcept* ValueError:\n  pass', 3, 8)
check('(0x+1)', 1, 3)
check('x = 0xI', 1, 6)
check('0010 + 2', 1, 1)
check('x = 32e-+4', 1, 8)
check('x = 0o9', 1, 7)
check('α = 0xI', 1, 6)
check(b'\xce\xb1 = 0xI', 1, 6)
check(b'# -*- coding: iso8859-7 -*-\n\xe1 = 0xI', 2, 6, encoding='iso8859-7')
check(b"if 1:\n            def foo():\n                '''\n\n            def bar():\n                pass\n\n            def baz():\n                '''quux'''\n            ", 9, 24)
check('pass\npass\npass\n(1+)\npass\npass\npass', 4, 4)
check('(1+)', 1, 4)
check('[interesting\nfoo()\n', 1, 1)
check(b"\xef\xbb\xbf#coding: utf8\nprint('\xe6\x88\x91')\n", 0, -1)
check("f'''\n            {\n            (123_a)\n            }'''", 3, 17)
check('f\'\'\'\n            {\n            f"""\n            {\n            (123_a)\n            }\n            """\n            }\'\'\'', 5, 17)
check('f"""\n\n\n            {\n            6\n            0="""', 5, 13)
check('b"fooжжж"'.encode(), 1, 1, 1, 10)
check('x = [(yield i) for i in range(3)]', 1, 7)
check('def f():\n  from _ import *', 2, 17)
check('def f(x, x):\n  pass', 1, 10)
check('{i for i in range(5) if (j := 0) for j in range(5)}', 1, 38)
check('def f(x):\n  nonlocal x', 2, 3)
check('def f(x):\n  x = 1\n  global x', 3, 3)
check('nonlocal x', 1, 1)
check('def f():\n  global x\n  nonlocal x', 2, 3)
check('from __future__ import doesnt_exist', 1, 24)
check('from __future__ import braces', 1, 24)
check('x=1\nfrom __future__ import division', 2, 1)
check('foo(1=2)', 1, 5)
check('def f():\n  x, y: int', 2, 3)
check('[*x for x in xs]', 1, 2)
check('foo(x for x in range(10), 100)', 1, 5)
check('for 1 in []: pass', 1, 5)
check('(yield i) = 2', 1, 2)
check('def f(*):\n  pass', 1, 7)
print("ExceptionTests::testSyntaxErrorOffset: ok")
