# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "re"
# dimension = "behavior"
# case = "re_tests__test_symbolic_groups_errors"
# subject = "cpython.test_re.ReTests.test_symbolic_groups_errors"
# kind = "semantic"
# xfail = "auto-ported CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_re.py"
# status = "filled"
# ///
# mamba-xfail: auto-ported CPython test; mamba promotion pending
# Auto-ported from CPython 3.12 test_re.py::ReTests::test_symbolic_groups_errors
"""Auto-ported test: ReTests::test_symbolic_groups_errors (CPython 3.12 oracle)."""


from test.support import gc_collect, bigmemtest, _2G, cpython_only, captured_stdout, check_disallow_instantiation, is_emscripten, is_wasi, SHORT_TIMEOUT, requires_resource
import locale
import re
import string
import sys
import time
import unittest
import warnings
from re import Scanner
from weakref import proxy


try:
    import _multiprocessing
except ImportError:
    multiprocessing = None
else:
    import multiprocessing

class S(str):

    def __getitem__(self, index):
        return S(super().__getitem__(index))

class B(bytes):

    def __getitem__(self, index):
        return B(super().__getitem__(index))

def get_debug_out(pat):
    with captured_stdout() as out:
        re.compile(pat, re.DEBUG)
    return out.getvalue()


# --- test body ---
LITERAL_CHARS = string.ascii_letters + string.digits + '!"%\',/:;<=>@_`'

def assertMatch(pattern, text, match=None, span=None, matcher=re.fullmatch):
    if match is None and span is None:
        match = text
        span = (0, len(text))
    elif match is None or span is None:
        raise ValueError('If match is not None, span should be specified (and vice versa).')
    m = matcher(pattern, text)

    assert m

    assert m.group() == match

    assert m.span() == span

def assertTypedEqual(actual, expect, msg=None):

    assert actual == expect

    def recurse(actual, expect):
        if isinstance(expect, (tuple, list)):
            for x, y in zip(actual, expect):
                recurse(x, y)
        else:
            self.assertIs(type(actual), type(expect), msg)
    recurse(actual, expect)

def bump_num(matchobj):
    int_value = int(matchobj.group(0))
    return str(int_value + 1)

def checkPatternError(pattern, errmsg, pos=None):
    try:
        re.compile(pattern)
        raise AssertionError('expected re.error')
    except re.error as _aR_e:
        import types as _types_aR
        cm = _types_aR.SimpleNamespace(exception=_aR_e)
    err = cm.exception

    assert err.msg == errmsg
    if pos is not None:

        assert err.pos == pos

def checkTemplateError(pattern, repl, string, errmsg, pos=None):
    try:
        re.sub(pattern, repl, string)
        raise AssertionError('expected re.error')
    except re.error as _aR_e:
        import types as _types_aR
        cm = _types_aR.SimpleNamespace(exception=_aR_e)
    err = cm.exception

    assert err.msg == errmsg
    if pos is not None:

        assert err.pos == pos

def check_en_US_iso88591():
    locale.setlocale(locale.LC_CTYPE, 'en_US.iso88591')

    assert re.match(b'\xc5\xe5', b'\xc5\xe5', re.L | re.I)

    assert re.match(b'\xc5', b'\xe5', re.L | re.I)

    assert re.match(b'\xe5', b'\xc5', re.L | re.I)

    assert re.match(b'(?Li)\xc5\xe5', b'\xc5\xe5')

    assert re.match(b'(?Li)\xc5', b'\xe5')

    assert re.match(b'(?Li)\xe5', b'\xc5')

def check_en_US_utf8():
    locale.setlocale(locale.LC_CTYPE, 'en_US.utf8')

    assert re.match(b'\xc5\xe5', b'\xc5\xe5', re.L | re.I)

    assert re.match(b'\xc5', b'\xe5', re.L | re.I) is None

    assert re.match(b'\xe5', b'\xc5', re.L | re.I) is None

    assert re.match(b'(?Li)\xc5\xe5', b'\xc5\xe5')

    assert re.match(b'(?Li)\xc5', b'\xe5') is None

    assert re.match(b'(?Li)\xe5', b'\xc5') is None

def check_interrupt(pattern, string, maxcount):

    class Interrupt(Exception):
        pass
    p = re.compile(pattern)
    for n in range(maxcount):
        try:
            p._fail_after(n, Interrupt)
            p.match(string)
            return n
        except Interrupt:
            pass
        finally:
            p._fail_after(-1, None)
checkPatternError('(?P<a>)(?P<a>)', "redefinition of group name 'a' as group 2; was group 1")
checkPatternError('(?P<a>(?P=a))', 'cannot refer to an open group', 10)
checkPatternError('(?Pxy)', 'unknown extension ?Px')
checkPatternError('(?P<a>)(?P=a', 'missing ), unterminated name', 11)
checkPatternError('(?P=', 'missing group name', 4)
checkPatternError('(?P=)', 'missing group name', 4)
checkPatternError('(?P=1)', "bad character in group name '1'", 4)
checkPatternError('(?P=a)', "unknown group name 'a'")
checkPatternError('(?P=a1)', "unknown group name 'a1'")
checkPatternError('(?P=a.)', "bad character in group name 'a.'", 4)
checkPatternError('(?P<)', 'missing >, unterminated name', 4)
checkPatternError('(?P<a', 'missing >, unterminated name', 4)
checkPatternError('(?P<', 'missing group name', 4)
checkPatternError('(?P<>)', 'missing group name', 4)
checkPatternError('(?P<1>)', "bad character in group name '1'", 4)
checkPatternError('(?P<a.>)', "bad character in group name 'a.'", 4)
checkPatternError('(?(', 'missing group name', 3)
checkPatternError('(?())', 'missing group name', 3)
checkPatternError('(?(a))', "unknown group name 'a'", 3)
checkPatternError('(?(-1))', "bad character in group name '-1'", 3)
checkPatternError('(?(1a))', "bad character in group name '1a'", 3)
checkPatternError('(?(a.))', "bad character in group name 'a.'", 3)
checkPatternError('(?P<©>x)', "bad character in group name '©'", 4)
checkPatternError('(?P=©)', "bad character in group name '©'", 4)
checkPatternError('(?(©)y)', "bad character in group name '©'", 3)
checkPatternError(b'(?P<\xc2\xb5>x)', "bad character in group name '\\xc2\\xb5'", 4)
checkPatternError(b'(?P=\xc2\xb5)', "bad character in group name '\\xc2\\xb5'", 4)
checkPatternError(b'(?(\xc2\xb5)y)', "bad character in group name '\\xc2\\xb5'", 3)
print("ReTests::test_symbolic_groups_errors: ok")
