# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "codeccallbacks"
# dimension = "behavior"
# case = "codec_callback_test__test_callbacks"
# subject = "cpython.test_codeccallbacks.CodecCallbackTest.test_callbacks"
# kind = "semantic"
# xfail = "auto-ported CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_codeccallbacks.py"
# status = "filled"
# ///
# mamba-xfail: auto-ported CPython test; mamba promotion pending
# Auto-ported from CPython 3.12 test_codeccallbacks.py::CodecCallbackTest::test_callbacks
"""Auto-ported test: CodecCallbackTest::test_callbacks (CPython 3.12 oracle)."""


import codecs
import html.entities
import itertools
import re
import sys
import unicodedata
import unittest


class PosReturn:

    def __init__(self):
        self.pos = 0

    def handle(self, exc):
        oldpos = self.pos
        realpos = oldpos
        if realpos < 0:
            realpos = len(exc.object) + realpos
        if realpos <= exc.start:
            self.pos = len(exc.object)
        return ('<?>', oldpos)

class RepeatedPosReturn:

    def __init__(self, repl='<?>'):
        self.repl = repl
        self.pos = 0
        self.count = 0

    def handle(self, exc):
        if self.count > 0:
            self.count -= 1
            return (self.repl, self.pos)
        return (self.repl, exc.end)

class BadStartUnicodeEncodeError(UnicodeEncodeError):

    def __init__(self):
        UnicodeEncodeError.__init__(self, 'ascii', '', 0, 1, 'bad')
        self.start = []

class BadObjectUnicodeEncodeError(UnicodeEncodeError):

    def __init__(self):
        UnicodeEncodeError.__init__(self, 'ascii', '', 0, 1, 'bad')
        self.object = []

class NoEndUnicodeDecodeError(UnicodeDecodeError):

    def __init__(self):
        UnicodeDecodeError.__init__(self, 'ascii', bytearray(b''), 0, 1, 'bad')
        del self.end

class BadObjectUnicodeDecodeError(UnicodeDecodeError):

    def __init__(self):
        UnicodeDecodeError.__init__(self, 'ascii', bytearray(b''), 0, 1, 'bad')
        self.object = []

class NoStartUnicodeTranslateError(UnicodeTranslateError):

    def __init__(self):
        UnicodeTranslateError.__init__(self, '', 0, 1, 'bad')
        del self.start

class NoEndUnicodeTranslateError(UnicodeTranslateError):

    def __init__(self):
        UnicodeTranslateError.__init__(self, '', 0, 1, 'bad')
        del self.end

class NoObjectUnicodeTranslateError(UnicodeTranslateError):

    def __init__(self):
        UnicodeTranslateError.__init__(self, '', 0, 1, 'bad')
        del self.object


# --- test body ---
def handler1(exc):
    r = range(exc.start, exc.end)
    if isinstance(exc, UnicodeEncodeError):
        l = ['<%d>' % ord(exc.object[pos]) for pos in r]
    elif isinstance(exc, UnicodeDecodeError):
        l = ['<%d>' % exc.object[pos] for pos in r]
    else:
        raise TypeError("don't know how to handle %r" % exc)
    return ('[%s]' % ''.join(l), exc.end)
codecs.register_error('test.handler1', handler1)

def handler2(exc):
    if not isinstance(exc, UnicodeDecodeError):
        raise TypeError("don't know how to handle %r" % exc)
    l = ['<%d>' % exc.object[pos] for pos in range(exc.start, exc.end)]
    return ('[%s]' % ''.join(l), exc.end + 1)
codecs.register_error('test.handler2', handler2)
s = b'\x00\x81\x7f\x80\xff'

assert s.decode('ascii', 'test.handler1') == '\x00[<129>]\x7f[<128>][<255>]'

assert s.decode('ascii', 'test.handler2') == '\x00[<129>][<128>]'

assert b'\\u3042\\u3xxx'.decode('unicode-escape', 'test.handler1') == 'あ[<92><117><51>]xxx'

assert b'\\u3042\\u3xx'.decode('unicode-escape', 'test.handler1') == 'あ[<92><117><51>]xx'

assert codecs.charmap_decode(b'abc', 'test.handler1', {ord('a'): 'z'})[0] == 'z[<98>][<99>]'

assert 'güßrk'.encode('ascii', 'test.handler1') == b'g[<252><223>]rk'

assert 'güß'.encode('ascii', 'test.handler1') == b'g[<252><223>]'
print("CodecCallbackTest::test_callbacks: ok")
