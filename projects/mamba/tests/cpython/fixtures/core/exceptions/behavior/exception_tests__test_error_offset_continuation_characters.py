# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "core"
# lib = "exceptions"
# dimension = "behavior"
# case = "exception_tests__test_error_offset_continuation_characters"
# subject = "cpython.test_exceptions.ExceptionTests.test_error_offset_continuation_characters"
# kind = "semantic"
# xfail = "auto-ported CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_exceptions.py"
# status = "filled"
# ///
# mamba-xfail: auto-ported CPython test; mamba promotion pending
# Auto-ported from CPython 3.12 test_exceptions.py::ExceptionTests::test_error_offset_continuation_characters
"""Auto-ported test: ExceptionTests::test_error_offset_continuation_characters (CPython 3.12 oracle)."""


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

def testAttributes():
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

def testChainingAttrs():
    e = Exception()

    assert e.__context__ is None

    assert e.__cause__ is None
    e = TypeError()

    assert e.__context__ is None

    assert e.__cause__ is None

    class MyException(OSError):
        pass
    e = MyException()

    assert e.__context__ is None

    assert e.__cause__ is None

def testChainingDescriptors():
    try:
        raise Exception()
    except Exception as exc:
        e = exc

    assert e.__context__ is None

    assert e.__cause__ is None

    assert not e.__suppress_context__
    e.__context__ = NameError()
    e.__cause__ = None

    assert isinstance(e.__context__, NameError)

    assert e.__cause__ is None

    assert e.__suppress_context__
    e.__suppress_context__ = False

    assert not e.__suppress_context__

def testExceptionCleanupState():

    class MyException(Exception):

        def __init__(self, obj):
            self.obj = obj

    class MyObj:
        pass

    def inner_raising_func():
        local_ref = obj
        raise MyException(obj)
    obj = MyObj()
    wr = weakref.ref(obj)
    try:
        inner_raising_func()
    except MyException as e:
        pass
    obj = None
    gc_collect()
    obj = wr()

    assert obj is None
    obj = MyObj()
    wr = weakref.ref(obj)
    try:
        inner_raising_func()
    except MyException:
        pass
    obj = None
    gc_collect()
    obj = wr()

    assert obj is None
    obj = MyObj()
    wr = weakref.ref(obj)
    try:
        inner_raising_func()
    except:
        pass
    obj = None
    gc_collect()
    obj = wr()

    assert obj is None
    obj = MyObj()
    wr = weakref.ref(obj)
    for i in [0]:
        try:
            inner_raising_func()
        except:
            break
    obj = None
    gc_collect()
    obj = wr()

    assert obj is None
    obj = MyObj()
    wr = weakref.ref(obj)
    try:
        try:
            inner_raising_func()
        except:
            raise KeyError
    except KeyError as e:
        e.__context__ = None
        obj = None
        gc_collect()
        obj = wr()
        if check_impl_detail(cpython=False):
            gc_collect()

        assert obj is None
    obj = MyObj()
    wr = weakref.ref(obj)
    try:
        inner_raising_func()
    except MyException:
        try:
            try:
                raise
            finally:
                raise
        except MyException:
            pass
    obj = None
    if check_impl_detail(cpython=False):
        gc_collect()
    obj = wr()

    assert obj is None

    class Context:

        def __enter__(self):
            return self

        def __exit__(self, exc_type, exc_value, exc_tb):
            return True
    obj = MyObj()
    wr = weakref.ref(obj)
    with Context():
        inner_raising_func()
    obj = None
    if check_impl_detail(cpython=False):
        gc_collect()
    obj = wr()

    assert obj is None

def testInfiniteRecursion():

    def f():
        return f()

    try:
        f()
        raise AssertionError('expected RecursionError')
    except RecursionError:
        pass

    def g():
        try:
            return g()
        except ValueError:
            return -1

    try:
        g()
        raise AssertionError('expected RecursionError')
    except RecursionError:
        pass

def testInvalidTraceback():
    try:
        Exception().__traceback__ = 5
    except TypeError as e:

        assert '__traceback__ must be a traceback' in str(e)
    else:

        raise AssertionError('No exception raised')

def testKeywordArgs():

    try:
        BaseException(a=1)
        raise AssertionError('expected TypeError')
    except TypeError:
        pass

    class DerivedException(BaseException):

        def __init__(self, fancy_arg):
            BaseException.__init__(self)
            self.fancy_arg = fancy_arg
    x = DerivedException(fancy_arg=42)

    assert x.fancy_arg == 42

def testMemoryErrorBigSource(size):
    src = b'if True:\n%*s' % (size, b'pass')
    try:
        compile(src, '<fragment>', 'exec')
        raise AssertionError('expected OverflowError')
    except OverflowError as _aR_e:
        import re as _re_aR
        assert _re_aR.search('Parser column offset overflow', str(_aR_e))

def testNoneClearsTracebackAttr():
    try:
        raise IndexError(4)
    except Exception as e:
        tb = e.__traceback__
    e = Exception()
    e.__traceback__ = tb
    e.__traceback__ = None

    assert e.__traceback__ == None

def testRaising():
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

def testSettingException():

    class BadException(Exception):

        def __init__(self_):
            raise RuntimeError("can't instantiate BadException")

    class InvalidException:
        pass

    def test_capi1():
        import _testcapi
        try:
            _testcapi.raise_exception(BadException, 1)
        except TypeError as err:
            co = err.__traceback__.tb_frame.f_code
            self.assertEqual(co.co_name, 'test_capi1')
            self.assertTrue(co.co_filename.endswith('test_exceptions.py'))
        else:
            self.fail('Expected exception')

    def test_capi2():
        import _testcapi
        try:
            _testcapi.raise_exception(BadException, 0)
        except RuntimeError as err:
            tb = err.__traceback__.tb_next
            co = tb.tb_frame.f_code
            self.assertEqual(co.co_name, '__init__')
            self.assertTrue(co.co_filename.endswith('test_exceptions.py'))
            co2 = tb.tb_frame.f_back.f_code
            self.assertEqual(co2.co_name, 'test_capi2')
        else:
            self.fail('Expected exception')

    def test_capi3():
        import _testcapi
        self.assertRaises(SystemError, _testcapi.raise_exception, InvalidException, 1)
    test_capi1()
    test_capi2()
    test_capi3()

def testSyntaxErrorMessage():

    def ckmsg(src, msg):
        with self.subTest(src=src, msg=msg):
            try:
                compile(src, '<fragment>', 'exec')
            except SyntaxError as e:
                if e.msg != msg:
                    self.fail('expected %s, got %s' % (msg, e.msg))
            else:
                self.fail('failed to get expected SyntaxError')
    s = 'if 1:\n        try:\n            continue\n        except:\n            pass'
    ckmsg(s, "'continue' not properly in loop")
    ckmsg('continue\n', "'continue' not properly in loop")
    ckmsg("f'{6 0}'", 'invalid syntax. Perhaps you forgot a comma?')

def testSyntaxErrorMissingParens():

    def ckmsg(src, msg, exception=SyntaxError):
        try:
            compile(src, '<fragment>', 'exec')
        except exception as e:
            if e.msg != msg:
                self.fail('expected %s, got %s' % (msg, e.msg))
        else:
            self.fail('failed to get expected SyntaxError')
    s = 'print "old style"'
    ckmsg(s, "Missing parentheses in call to 'print'. Did you mean print(...)?")
    s = 'print "old style",'
    ckmsg(s, "Missing parentheses in call to 'print'. Did you mean print(...)?")
    s = 'print f(a+b,c)'
    ckmsg(s, "Missing parentheses in call to 'print'. Did you mean print(...)?")
    s = 'exec "old style"'
    ckmsg(s, "Missing parentheses in call to 'exec'. Did you mean exec(...)?")
    s = 'exec f(a+b,c)'
    ckmsg(s, "Missing parentheses in call to 'exec'. Did you mean exec(...)?")
    s = 'print (a+b,c) $ 42'
    ckmsg(s, 'invalid syntax')
    s = 'exec (a+b,c) $ 42'
    ckmsg(s, 'invalid syntax')
    s = 'if True:\nprint "No indent"'
    ckmsg(s, "expected an indented block after 'if' statement on line 1", IndentationError)
    s = 'if True:\n        print()\n\texec "mixed tabs and spaces"'
    ckmsg(s, 'inconsistent use of tabs and spaces in indentation', TabError)

def testSyntaxErrorOffset():
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

def testWithTraceback():
    try:
        raise IndexError(4)
    except Exception as e:
        tb = e.__traceback__
    e = BaseException().with_traceback(tb)

    assert isinstance(e, BaseException)

    assert e.__traceback__ == tb
    e = IndexError(5).with_traceback(tb)

    assert isinstance(e, IndexError)

    assert e.__traceback__ == tb

    class MyException(Exception):
        pass
    e = MyException().with_traceback(tb)

    assert isinstance(e, MyException)

    assert e.__traceback__ == tb
check = check
check('"\\\n"(1 for c in I,\\\n\\', 2, 2)
print("ExceptionTests::test_error_offset_continuation_characters: ok")
