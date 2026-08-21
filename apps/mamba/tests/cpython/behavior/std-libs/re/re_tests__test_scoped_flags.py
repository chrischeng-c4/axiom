# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "re"
# dimension = "behavior"
# case = "re_tests__test_scoped_flags"
# subject = "cpython.test_re.ReTests.test_scoped_flags"
# kind = "semantic"
# xfail = "auto-ported CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_re.py"
# status = "filled"
# ///
# mamba-xfail: auto-ported CPython test; mamba promotion pending
# Auto-ported from CPython 3.12 test_re.py::ReTests::test_scoped_flags
"""Auto-ported test: ReTests::test_scoped_flags (CPython 3.12 oracle)."""


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

assert re.match('(?i:a)b', 'Ab')

assert re.match('(?i:a)b', 'aB') is None

assert re.match('(?-i:a)b', 'Ab', re.IGNORECASE) is None

assert re.match('(?-i:a)b', 'aB', re.IGNORECASE)

assert re.match('(?i:(?-i:a)b)', 'Ab') is None

assert re.match('(?i:(?-i:a)b)', 'aB')

assert re.match('\\w(?a:\\W)\\w', 'ààà')

assert re.match('(?a:\\W(?u:\\w)\\W)', 'ààà')

assert re.match('\\W(?u:\\w)\\W', 'ààà', re.ASCII)
checkPatternError('(?a)(?-a:\\w)', "bad inline flags: cannot turn off flags 'a', 'u' and 'L'", 8)
checkPatternError('(?i-i:a)', 'bad inline flags: flag turned on and off', 5)
checkPatternError('(?au:a)', "bad inline flags: flags 'a', 'u' and 'L' are incompatible", 4)
checkPatternError(b'(?aL:a)', "bad inline flags: flags 'a', 'u' and 'L' are incompatible", 4)
checkPatternError('(?-', 'missing flag', 3)
checkPatternError('(?-+', 'missing flag', 3)
checkPatternError('(?-z', 'unknown flag', 3)
checkPatternError('(?-i', 'missing :', 4)
checkPatternError('(?-i)', 'missing :', 4)
checkPatternError('(?-i+', 'missing :', 4)
checkPatternError('(?-iz', 'unknown flag', 4)
checkPatternError('(?i:', 'missing ), unterminated subpattern', 0)
checkPatternError('(?i', 'missing -, : or )', 3)
checkPatternError('(?i+', 'missing -, : or )', 3)
checkPatternError('(?iz', 'unknown flag', 3)
print("ReTests::test_scoped_flags: ok")
