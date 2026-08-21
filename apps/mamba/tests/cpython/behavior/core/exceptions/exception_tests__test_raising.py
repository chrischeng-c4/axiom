# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "core"
# lib = "exceptions"
# dimension = "behavior"
# case = "exception_tests__test_raising"
# subject = "cpython.test_exceptions.ExceptionTests.testRaising"
# kind = "semantic"
# xfail = "auto-ported CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_exceptions.py"
# status = "filled"
# ///
# mamba-xfail: auto-ported CPython test; mamba promotion pending
# Auto-ported from CPython 3.12 test_exceptions.py::ExceptionTests::testRaising
"""Auto-ported test: ExceptionTests::testRaising (CPython 3.12 oracle)."""


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
raise_catch(AttributeError, 'AttributeError')

try:
    getattr(sys, 'undefined_attribute')
    raise AssertionError('expected AttributeError')
except AttributeError:
    pass
raise_catch(EOFError, 'EOFError')
fp = open(TESTFN, 'w', encoding='utf-8')
fp.close()
fp = open(TESTFN, 'r', encoding='utf-8')
savestdin = sys.stdin
try:
    try:
        import marshal
        marshal.loads(b'')
    except EOFError:
        pass
finally:
    sys.stdin = savestdin
    fp.close()
    unlink(TESTFN)
raise_catch(OSError, 'OSError')

try:
    open('this file does not exist', 'r')
    raise AssertionError('expected OSError')
except OSError:
    pass
raise_catch(ImportError, 'ImportError')

try:
    __import__('undefined_module')
    raise AssertionError('expected ImportError')
except ImportError:
    pass
raise_catch(IndexError, 'IndexError')
x = []

try:
    x.__getitem__(10)
    raise AssertionError('expected IndexError')
except IndexError:
    pass
raise_catch(KeyError, 'KeyError')
x = {}

try:
    x.__getitem__('key')
    raise AssertionError('expected KeyError')
except KeyError:
    pass
raise_catch(KeyboardInterrupt, 'KeyboardInterrupt')
raise_catch(MemoryError, 'MemoryError')
raise_catch(NameError, 'NameError')
try:
    x = undefined_variable
except NameError:
    pass
raise_catch(OverflowError, 'OverflowError')
x = 1
for dummy in range(128):
    x += x
raise_catch(RuntimeError, 'RuntimeError')
raise_catch(RecursionError, 'RecursionError')
raise_catch(SyntaxError, 'SyntaxError')
try:
    exec('/\n')
except SyntaxError:
    pass
raise_catch(IndentationError, 'IndentationError')
raise_catch(TabError, 'TabError')
try:
    compile('try:\n\t1/0\n    \t1/0\nfinally:\n pass\n', '<string>', 'exec')
except TabError:
    pass
else:

    raise AssertionError('TabError not raised')
raise_catch(SystemError, 'SystemError')
raise_catch(SystemExit, 'SystemExit')

try:
    sys.exit(0)
    raise AssertionError('expected SystemExit')
except SystemExit:
    pass
raise_catch(TypeError, 'TypeError')
try:
    [] + ()
except TypeError:
    pass
raise_catch(ValueError, 'ValueError')

try:
    chr(17 << 16)
    raise AssertionError('expected ValueError')
except ValueError:
    pass
raise_catch(ZeroDivisionError, 'ZeroDivisionError')
try:
    x = 1 / 0
except ZeroDivisionError:
    pass
raise_catch(Exception, 'Exception')
try:
    x = 1 / 0
except Exception as e:
    pass
raise_catch(StopAsyncIteration, 'StopAsyncIteration')
print("ExceptionTests::testRaising: ok")
