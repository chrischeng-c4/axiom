use super::super::super::super::harness::*;

/// Ported from `tests/cpython/behavior/core/exceptions/exception_tests__test_chaining_attrs.py`.
#[test]
fn test_gen_behavior_core_exceptions_exception_tests__test_chaining_attrs() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "core"
# lib = "exceptions"
# dimension = "behavior"
# case = "exception_tests__test_chaining_attrs"
# subject = "cpython.test_exceptions.ExceptionTests.testChainingAttrs"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_exceptions.py"
# status = "filled"
# ///
# Auto-ported from CPython 3.12 test_exceptions.py::ExceptionTests::testChainingAttrs
"""Auto-ported test: ExceptionTests::testChainingAttrs (CPython 3.12 oracle)."""


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
print("ExceptionTests::testChainingAttrs: ok")
"###);
    assert_output(&out, r###"ExceptionTests::testChainingAttrs: ok
"###);
}

/// Ported from `tests/cpython/behavior/core/exceptions/exception_tests__test_generator_finalizing_and_sys_exception.py`.
#[test]
fn test_gen_behavior_core_exceptions_exception_tests__test_generator_finalizing_and_sys_exception() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "core"
# lib = "exceptions"
# dimension = "behavior"
# case = "exception_tests__test_generator_finalizing_and_sys_exception"
# subject = "cpython.test_exceptions.ExceptionTests.test_generator_finalizing_and_sys_exception"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_exceptions.py"
# status = "filled"
# ///
# Auto-ported from CPython 3.12 test_exceptions.py::ExceptionTests::test_generator_finalizing_and_sys_exception
"""Auto-ported test: ExceptionTests::test_generator_finalizing_and_sys_exception (CPython 3.12 oracle)."""


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
def simple_gen():
    yield 1

def run_gen():
    gen = simple_gen()
    try:
        raise RuntimeError
    except RuntimeError:
        return next(gen)
run_gen()
gc_collect()

assert sys.exception() is None
print("ExceptionTests::test_generator_finalizing_and_sys_exception: ok")
"###);
    assert_output(&out, r###"ExceptionTests::test_generator_finalizing_and_sys_exception: ok
"###);
}

/// Ported from `tests/cpython/behavior/core/exceptions/exception_tests__test_generator_leaking2.py`.
#[test]
fn test_gen_behavior_core_exceptions_exception_tests__test_generator_leaking2() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "core"
# lib = "exceptions"
# dimension = "behavior"
# case = "exception_tests__test_generator_leaking2"
# subject = "cpython.test_exceptions.ExceptionTests.test_generator_leaking2"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_exceptions.py"
# status = "filled"
# ///
# Auto-ported from CPython 3.12 test_exceptions.py::ExceptionTests::test_generator_leaking2
"""Auto-ported test: ExceptionTests::test_generator_leaking2 (CPython 3.12 oracle)."""


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
def g():
    yield
try:
    raise RuntimeError
except RuntimeError:
    it = g()
    next(it)
try:
    next(it)
except StopIteration:
    pass

assert sys.exception() is None
print("ExceptionTests::test_generator_leaking2: ok")
"###);
    assert_output(&out, r###"ExceptionTests::test_generator_leaking2: ok
"###);
}

/// Ported from `tests/cpython/behavior/core/exceptions/exception_tests__test_no_hang_on_context_chain_cycle3.py`.
#[test]
fn test_gen_behavior_core_exceptions_exception_tests__test_no_hang_on_context_chain_cycle3() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "core"
# lib = "exceptions"
# dimension = "behavior"
# case = "exception_tests__test_no_hang_on_context_chain_cycle3"
# subject = "cpython.test_exceptions.ExceptionTests.test_no_hang_on_context_chain_cycle3"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_exceptions.py"
# status = "filled"
# ///
# Auto-ported from CPython 3.12 test_exceptions.py::ExceptionTests::test_no_hang_on_context_chain_cycle3
"""Auto-ported test: ExceptionTests::test_no_hang_on_context_chain_cycle3 (CPython 3.12 oracle)."""


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
class A(Exception):
    pass

class B(Exception):
    pass

class C(Exception):
    pass

class D(Exception):
    pass

class E(Exception):
    pass
try:
    try:
        raise A()
    except A as _a:
        a = _a
        try:
            raise B()
        except B as _b:
            b = _b
            try:
                raise C()
            except C as _c:
                c = _c
                a.__context__ = c
                try:
                    raise D()
                except D as _d:
                    d = _d
                    e = E()
                    raise e
    raise AssertionError('expected E')
except E as _aR_e:
    import types as _types_aR
    cm = _types_aR.SimpleNamespace(exception=_aR_e)

assert cm.exception is e

assert e.__context__ is d

assert d.__context__ is c

assert c.__context__ is b

assert b.__context__ is a

assert a.__context__ is c
print("ExceptionTests::test_no_hang_on_context_chain_cycle3: ok")
"###);
    assert_output(&out, r###"ExceptionTests::test_no_hang_on_context_chain_cycle3: ok
"###);
}

/// Ported from `tests/cpython/behavior/core/exceptions/exception_tests__test_none_clears_traceback_attr.py`.
#[test]
fn test_gen_behavior_core_exceptions_exception_tests__test_none_clears_traceback_attr() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "core"
# lib = "exceptions"
# dimension = "behavior"
# case = "exception_tests__test_none_clears_traceback_attr"
# subject = "cpython.test_exceptions.ExceptionTests.testNoneClearsTracebackAttr"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_exceptions.py"
# status = "filled"
# ///
# Auto-ported from CPython 3.12 test_exceptions.py::ExceptionTests::testNoneClearsTracebackAttr
"""Auto-ported test: ExceptionTests::testNoneClearsTracebackAttr (CPython 3.12 oracle)."""


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
try:
    raise IndexError(4)
except Exception as e:
    tb = e.__traceback__
e = Exception()
e.__traceback__ = tb
e.__traceback__ = None

assert e.__traceback__ == None
print("ExceptionTests::testNoneClearsTracebackAttr: ok")
"###);
    assert_output(&out, r###"ExceptionTests::testNoneClearsTracebackAttr: ok
"###);
}

/// Ported from `tests/cpython/behavior/core/exceptions/exception_tests__test_str.py`.
#[test]
fn test_gen_behavior_core_exceptions_exception_tests__test_str() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "core"
# lib = "exceptions"
# dimension = "behavior"
# case = "exception_tests__test_str"
# subject = "cpython.test_exceptions.ExceptionTests.test_str"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_exceptions.py"
# status = "filled"
# ///
# Auto-ported from CPython 3.12 test_exceptions.py::ExceptionTests::test_str
"""Auto-ported test: ExceptionTests::test_str (CPython 3.12 oracle)."""


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

assert str(Exception)

assert str(Exception('a'))

assert str(Exception('a', 'b'))
print("ExceptionTests::test_str: ok")
"###);
    assert_output(&out, r###"ExceptionTests::test_str: ok
"###);
}

/// Ported from `tests/cpython/behavior/core/exceptions/exception_tests__test_trashcan_recursion.py`.
#[test]
fn test_gen_behavior_core_exceptions_exception_tests__test_trashcan_recursion() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "core"
# lib = "exceptions"
# dimension = "behavior"
# case = "exception_tests__test_trashcan_recursion"
# subject = "cpython.test_exceptions.ExceptionTests.test_trashcan_recursion"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_exceptions.py"
# status = "filled"
# ///
# Auto-ported from CPython 3.12 test_exceptions.py::ExceptionTests::test_trashcan_recursion
"""Auto-ported test: ExceptionTests::test_trashcan_recursion (CPython 3.12 oracle)."""


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
def foo():
    o = object()
    for x in range(1000000):
        o = o.__dir__
foo()
support.gc_collect()
print("ExceptionTests::test_trashcan_recursion: ok")
"###);
    assert_output(&out, r###"ExceptionTests::test_trashcan_recursion: ok
"###);
}

/// Ported from `tests/cpython/behavior/core/exceptions/exception_tests__test_yield_in_nested_try_excepts.py`.
#[test]
fn test_gen_behavior_core_exceptions_exception_tests__test_yield_in_nested_try_excepts() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "core"
# lib = "exceptions"
# dimension = "behavior"
# case = "exception_tests__test_yield_in_nested_try_excepts"
# subject = "cpython.test_exceptions.ExceptionTests.test_yield_in_nested_try_excepts"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_exceptions.py"
# status = "filled"
# ///
# Auto-ported from CPython 3.12 test_exceptions.py::ExceptionTests::test_yield_in_nested_try_excepts
"""Auto-ported test: ExceptionTests::test_yield_in_nested_try_excepts (CPython 3.12 oracle)."""


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
class MainError(Exception):
    pass

class SubError(Exception):
    pass

def main():
    try:
        raise MainError()
    except MainError:
        try:
            yield
        except SubError:
            pass
        raise
coro = main()
coro.send(None)
try:
    coro.throw(SubError())
    raise AssertionError('expected MainError')
except MainError:
    pass
print("ExceptionTests::test_yield_in_nested_try_excepts: ok")
"###);
    assert_output(&out, r###"ExceptionTests::test_yield_in_nested_try_excepts: ok
"###);
}

/// Ported from `tests/cpython/behavior/core/exceptions/import_error_tests__test_non_str_argument.py`.
#[test]
fn test_gen_behavior_core_exceptions_import_error_tests__test_non_str_argument() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "core"
# lib = "exceptions"
# dimension = "behavior"
# case = "import_error_tests__test_non_str_argument"
# subject = "cpython.test_exceptions.ImportErrorTests.test_non_str_argument"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_exceptions.py"
# status = "filled"
# ///
# Auto-ported from CPython 3.12 test_exceptions.py::ImportErrorTests::test_non_str_argument
"""Auto-ported test: ImportErrorTests::test_non_str_argument (CPython 3.12 oracle)."""


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
with check_warnings(('', BytesWarning), quiet=True):
    arg = b'abc'
    exc = ImportError(arg)

    assert str(arg) == str(exc)
print("ImportErrorTests::test_non_str_argument: ok")
"###);
    assert_output(&out, r###"ImportErrorTests::test_non_str_argument: ok
"###);
}
