use super::super::super::super::harness::*;

/// Ported from `tests/cpython/behavior/std-libs/raise/test_context__test_c_exception_context.py`.
#[test]
fn test_gen_behavior_std_libs_raise_test_context__test_c_exception_context() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "raise"
# dimension = "behavior"
# case = "test_context__test_c_exception_context"
# subject = "cpython.test_raise.TestContext.test_c_exception_context"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_raise.py"
# status = "filled"
# ///
# Auto-ported from CPython 3.12 test_raise.py::TestContext::test_c_exception_context
"""Auto-ported test: TestContext::test_c_exception_context (CPython 3.12 oracle)."""


from test import support
import sys
import types
import unittest


'Tests for the raise statement.'

def get_tb():
    try:
        raise OSError()
    except OSError as e:
        return e.__traceback__

class Context:

    def __enter__(self):
        return self

    def __exit__(self, exc_type, exc_value, exc_tb):
        return True


# --- test body ---
try:
    try:
        1 / 0
    except:
        raise OSError
except OSError as e:

    assert isinstance(e.__context__, ZeroDivisionError)
else:

    raise AssertionError('No exception raised')
print("TestContext::test_c_exception_context: ok")
"###);
    assert_output(&out, r###"TestContext::test_c_exception_context: ok
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/raise/test_context__test_noraise_finally.py`.
#[test]
fn test_gen_behavior_std_libs_raise_test_context__test_noraise_finally() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "raise"
# dimension = "behavior"
# case = "test_context__test_noraise_finally"
# subject = "cpython.test_raise.TestContext.test_noraise_finally"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_raise.py"
# status = "filled"
# ///
# Auto-ported from CPython 3.12 test_raise.py::TestContext::test_noraise_finally
"""Auto-ported test: TestContext::test_noraise_finally (CPython 3.12 oracle)."""


from test import support
import sys
import types
import unittest


'Tests for the raise statement.'

def get_tb():
    try:
        raise OSError()
    except OSError as e:
        return e.__traceback__

class Context:

    def __enter__(self):
        return self

    def __exit__(self, exc_type, exc_value, exc_tb):
        return True


# --- test body ---
try:
    try:
        pass
    finally:
        raise OSError
except OSError as e:

    assert e.__context__ is None
else:

    raise AssertionError('No exception raised')
print("TestContext::test_noraise_finally: ok")
"###);
    assert_output(&out, r###"TestContext::test_noraise_finally: ok
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/raise/test_raise__test_except_reraise.py`.
#[test]
fn test_gen_behavior_std_libs_raise_test_raise__test_except_reraise() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "raise"
# dimension = "behavior"
# case = "test_raise__test_except_reraise"
# subject = "cpython.test_raise.TestRaise.test_except_reraise"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_raise.py"
# status = "filled"
# ///
# Auto-ported from CPython 3.12 test_raise.py::TestRaise::test_except_reraise
"""Auto-ported test: TestRaise::test_except_reraise (CPython 3.12 oracle)."""


from test import support
import sys
import types
import unittest


'Tests for the raise statement.'

def get_tb():
    try:
        raise OSError()
    except OSError as e:
        return e.__traceback__

class Context:

    def __enter__(self):
        return self

    def __exit__(self, exc_type, exc_value, exc_tb):
        return True


# --- test body ---
def reraise():
    try:
        raise TypeError('foo')
    except:
        try:
            raise KeyError('caught')
        except KeyError:
            pass
        raise

try:
    reraise()
    raise AssertionError('expected TypeError')
except TypeError:
    pass
print("TestRaise::test_except_reraise: ok")
"###);
    assert_output(&out, r###"TestRaise::test_except_reraise: ok
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/raise/test_raise__test_raise_from_none.py`.
#[test]
fn test_gen_behavior_std_libs_raise_test_raise__test_raise_from_none() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "raise"
# dimension = "behavior"
# case = "test_raise__test_raise_from_none"
# subject = "cpython.test_raise.TestRaise.test_raise_from_None"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_raise.py"
# status = "filled"
# ///
# Auto-ported from CPython 3.12 test_raise.py::TestRaise::test_raise_from_None
"""Auto-ported test: TestRaise::test_raise_from_None (CPython 3.12 oracle)."""


from test import support
import sys
import types
import unittest


'Tests for the raise statement.'

def get_tb():
    try:
        raise OSError()
    except OSError as e:
        return e.__traceback__

class Context:

    def __enter__(self):
        return self

    def __exit__(self, exc_type, exc_value, exc_tb):
        return True


# --- test body ---
try:
    try:
        raise TypeError('foo')
    except:
        raise ValueError() from None
except ValueError as e:

    assert isinstance(e.__context__, TypeError)

    assert e.__cause__ is None
print("TestRaise::test_raise_from_None: ok")
"###);
    assert_output(&out, r###"TestRaise::test_raise_from_None: ok
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/raise/test_raise__test_reraise.py`.
#[test]
fn test_gen_behavior_std_libs_raise_test_raise__test_reraise() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "raise"
# dimension = "behavior"
# case = "test_raise__test_reraise"
# subject = "cpython.test_raise.TestRaise.test_reraise"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_raise.py"
# status = "filled"
# ///
# Auto-ported from CPython 3.12 test_raise.py::TestRaise::test_reraise
"""Auto-ported test: TestRaise::test_reraise (CPython 3.12 oracle)."""


from test import support
import sys
import types
import unittest


'Tests for the raise statement.'

def get_tb():
    try:
        raise OSError()
    except OSError as e:
        return e.__traceback__

class Context:

    def __enter__(self):
        return self

    def __exit__(self, exc_type, exc_value, exc_tb):
        return True


# --- test body ---
try:
    try:
        raise IndexError()
    except IndexError as e:
        exc1 = e
        raise
except IndexError as exc2:

    assert exc1 is exc2
else:

    raise AssertionError('No exception raised')
print("TestRaise::test_reraise: ok")
"###);
    assert_output(&out, r###"TestRaise::test_reraise: ok
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/raise/test_raise__test_with_reraise1.py`.
#[test]
fn test_gen_behavior_std_libs_raise_test_raise__test_with_reraise1() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "raise"
# dimension = "behavior"
# case = "test_raise__test_with_reraise1"
# subject = "cpython.test_raise.TestRaise.test_with_reraise1"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_raise.py"
# status = "filled"
# ///
# Auto-ported from CPython 3.12 test_raise.py::TestRaise::test_with_reraise1
"""Auto-ported test: TestRaise::test_with_reraise1 (CPython 3.12 oracle)."""


from test import support
import sys
import types
import unittest


'Tests for the raise statement.'

def get_tb():
    try:
        raise OSError()
    except OSError as e:
        return e.__traceback__

class Context:

    def __enter__(self):
        return self

    def __exit__(self, exc_type, exc_value, exc_tb):
        return True


# --- test body ---
def reraise():
    try:
        raise TypeError('foo')
    except:
        with Context():
            pass
        raise

try:
    reraise()
    raise AssertionError('expected TypeError')
except TypeError:
    pass
print("TestRaise::test_with_reraise1: ok")
"###);
    assert_output(&out, r###"TestRaise::test_with_reraise1: ok
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/raise/test_raise__test_with_reraise2.py`.
#[test]
fn test_gen_behavior_std_libs_raise_test_raise__test_with_reraise2() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "raise"
# dimension = "behavior"
# case = "test_raise__test_with_reraise2"
# subject = "cpython.test_raise.TestRaise.test_with_reraise2"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_raise.py"
# status = "filled"
# ///
# Auto-ported from CPython 3.12 test_raise.py::TestRaise::test_with_reraise2
"""Auto-ported test: TestRaise::test_with_reraise2 (CPython 3.12 oracle)."""


from test import support
import sys
import types
import unittest


'Tests for the raise statement.'

def get_tb():
    try:
        raise OSError()
    except OSError as e:
        return e.__traceback__

class Context:

    def __enter__(self):
        return self

    def __exit__(self, exc_type, exc_value, exc_tb):
        return True


# --- test body ---
def reraise():
    try:
        raise TypeError('foo')
    except:
        with Context():
            raise KeyError('caught')
        raise

try:
    reraise()
    raise AssertionError('expected TypeError')
except TypeError:
    pass
print("TestRaise::test_with_reraise2: ok")
"###);
    assert_output(&out, r###"TestRaise::test_with_reraise2: ok
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/raise/test_raise__test_yield_reraise.py`.
#[test]
fn test_gen_behavior_std_libs_raise_test_raise__test_yield_reraise() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "raise"
# dimension = "behavior"
# case = "test_raise__test_yield_reraise"
# subject = "cpython.test_raise.TestRaise.test_yield_reraise"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_raise.py"
# status = "filled"
# ///
# Auto-ported from CPython 3.12 test_raise.py::TestRaise::test_yield_reraise
"""Auto-ported test: TestRaise::test_yield_reraise (CPython 3.12 oracle)."""


from test import support
import sys
import types
import unittest


'Tests for the raise statement.'

def get_tb():
    try:
        raise OSError()
    except OSError as e:
        return e.__traceback__

class Context:

    def __enter__(self):
        return self

    def __exit__(self, exc_type, exc_value, exc_tb):
        return True


# --- test body ---
def reraise():
    try:
        raise TypeError('foo')
    except:
        yield 1
        raise
g = reraise()
next(g)

try:
    (lambda: next(g))()
    raise AssertionError('expected TypeError')
except TypeError:
    pass

try:
    (lambda: next(g))()
    raise AssertionError('expected StopIteration')
except StopIteration:
    pass
print("TestRaise::test_yield_reraise: ok")
"###);
    assert_output(&out, r###"TestRaise::test_yield_reraise: ok
"###);
}
