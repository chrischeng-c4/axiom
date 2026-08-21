# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "core"
# lib = "exceptions"
# dimension = "behavior"
# case = "exception_tests__test_attributes"
# subject = "cpython.test_exceptions.ExceptionTests.testAttributes"
# kind = "semantic"
# xfail = "auto-ported CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_exceptions.py"
# status = "filled"
# ///
# mamba-xfail: auto-ported CPython test; mamba promotion pending
# Auto-ported from CPython 3.12 test_exceptions.py::ExceptionTests::testAttributes
"""Auto-ported test: ExceptionTests::testAttributes (CPython 3.12 oracle)."""


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
exceptionList = [(BaseException, (), {}, {'args': ()}), (BaseException, (1,), {}, {'args': (1,)}), (BaseException, ('foo',), {}, {'args': ('foo',)}), (BaseException, ('foo', 1), {}, {'args': ('foo', 1)}), (SystemExit, ('foo',), {}, {'args': ('foo',), 'code': 'foo'}), (OSError, ('foo',), {}, {'args': ('foo',), 'filename': None, 'filename2': None, 'errno': None, 'strerror': None}), (OSError, ('foo', 'bar'), {}, {'args': ('foo', 'bar'), 'filename': None, 'filename2': None, 'errno': 'foo', 'strerror': 'bar'}), (OSError, ('foo', 'bar', 'baz'), {}, {'args': ('foo', 'bar'), 'filename': 'baz', 'filename2': None, 'errno': 'foo', 'strerror': 'bar'}), (OSError, ('foo', 'bar', 'baz', None, 'quux'), {}, {'args': ('foo', 'bar'), 'filename': 'baz', 'filename2': 'quux'}), (OSError, ('errnoStr', 'strErrorStr', 'filenameStr'), {}, {'args': ('errnoStr', 'strErrorStr'), 'strerror': 'strErrorStr', 'errno': 'errnoStr', 'filename': 'filenameStr'}), (OSError, (1, 'strErrorStr', 'filenameStr'), {}, {'args': (1, 'strErrorStr'), 'errno': 1, 'strerror': 'strErrorStr', 'filename': 'filenameStr', 'filename2': None}), (SyntaxError, (), {}, {'msg': None, 'text': None, 'filename': None, 'lineno': None, 'offset': None, 'end_offset': None, 'print_file_and_line': None}), (SyntaxError, ('msgStr',), {}, {'args': ('msgStr',), 'text': None, 'print_file_and_line': None, 'msg': 'msgStr', 'filename': None, 'lineno': None, 'offset': None, 'end_offset': None}), (SyntaxError, ('msgStr', ('filenameStr', 'linenoStr', 'offsetStr', 'textStr', 'endLinenoStr', 'endOffsetStr')), {}, {'offset': 'offsetStr', 'text': 'textStr', 'args': ('msgStr', ('filenameStr', 'linenoStr', 'offsetStr', 'textStr', 'endLinenoStr', 'endOffsetStr')), 'print_file_and_line': None, 'msg': 'msgStr', 'filename': 'filenameStr', 'lineno': 'linenoStr', 'end_lineno': 'endLinenoStr', 'end_offset': 'endOffsetStr'}), (SyntaxError, ('msgStr', 'filenameStr', 'linenoStr', 'offsetStr', 'textStr', 'endLinenoStr', 'endOffsetStr', 'print_file_and_lineStr'), {}, {'text': None, 'args': ('msgStr', 'filenameStr', 'linenoStr', 'offsetStr', 'textStr', 'endLinenoStr', 'endOffsetStr', 'print_file_and_lineStr'), 'print_file_and_line': None, 'msg': 'msgStr', 'filename': None, 'lineno': None, 'offset': None, 'end_lineno': None, 'end_offset': None}), (UnicodeError, (), {}, {'args': ()}), (UnicodeEncodeError, ('ascii', 'a', 0, 1, 'ordinal not in range'), {}, {'args': ('ascii', 'a', 0, 1, 'ordinal not in range'), 'encoding': 'ascii', 'object': 'a', 'start': 0, 'reason': 'ordinal not in range'}), (UnicodeDecodeError, ('ascii', bytearray(b'\xff'), 0, 1, 'ordinal not in range'), {}, {'args': ('ascii', bytearray(b'\xff'), 0, 1, 'ordinal not in range'), 'encoding': 'ascii', 'object': b'\xff', 'start': 0, 'reason': 'ordinal not in range'}), (UnicodeDecodeError, ('ascii', b'\xff', 0, 1, 'ordinal not in range'), {}, {'args': ('ascii', b'\xff', 0, 1, 'ordinal not in range'), 'encoding': 'ascii', 'object': b'\xff', 'start': 0, 'reason': 'ordinal not in range'}), (UnicodeTranslateError, ('あ', 0, 1, 'ouch'), {}, {'args': ('あ', 0, 1, 'ouch'), 'object': 'あ', 'reason': 'ouch', 'start': 0, 'end': 1}), (NaiveException, ('foo',), {}, {'args': ('foo',), 'x': 'foo'}), (SlottedNaiveException, ('foo',), {}, {'args': ('foo',), 'x': 'foo'}), (AttributeError, ('foo',), dict(name='name', obj='obj'), dict(args=('foo',), name='name', obj='obj'))]
try:
    exceptionList.append((WindowsError, (1, 'strErrorStr', 'filenameStr'), {}, {'args': (1, 'strErrorStr'), 'strerror': 'strErrorStr', 'winerror': None, 'errno': 1, 'filename': 'filenameStr', 'filename2': None}))
except NameError:
    pass
for exc, args, kwargs, expected in exceptionList:
    try:
        e = exc(*args, **kwargs)
    except:
        print(f'\nexc={exc!r}, args={args!r}', file=sys.stderr)
    else:
        if not type(e).__name__.endswith('NaiveException'):

            assert type(e).__module__ == 'builtins'
        s = str(e)
        for checkArgName in expected:
            value = getattr(e, checkArgName)

            assert repr(value) == repr(expected[checkArgName])
        for p in [pickle]:
            for protocol in range(p.HIGHEST_PROTOCOL + 1):
                s = p.dumps(e, protocol)
                new = p.loads(s)
                for checkArgName in expected:
                    got = repr(getattr(new, checkArgName))
                    if exc == AttributeError and checkArgName == 'obj':
                        want = repr(None)
                    else:
                        want = repr(expected[checkArgName])

                    assert got == want
print("ExceptionTests::testAttributes: ok")
