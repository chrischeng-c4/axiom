# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "codeccallbacks"
# dimension = "behavior"
# case = "codec_callback_test__test_badandgoodxmlcharrefreplaceexceptions"
# subject = "cpython.test_codeccallbacks.CodecCallbackTest.test_badandgoodxmlcharrefreplaceexceptions"
# kind = "semantic"
# xfail = "auto-ported CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_codeccallbacks.py"
# status = "filled"
# ///
# mamba-xfail: auto-ported CPython test; mamba promotion pending
# Auto-ported from CPython 3.12 test_codeccallbacks.py::CodecCallbackTest::test_badandgoodxmlcharrefreplaceexceptions
"""Auto-ported test: CodecCallbackTest::test_badandgoodxmlcharrefreplaceexceptions (CPython 3.12 oracle)."""


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

try:
    codecs.xmlcharrefreplace_errors(42)
    raise AssertionError('expected TypeError')
except TypeError:
    pass

try:
    codecs.xmlcharrefreplace_errors(UnicodeError('ouch'))
    raise AssertionError('expected TypeError')
except TypeError:
    pass

try:
    codecs.xmlcharrefreplace_errors(UnicodeDecodeError('ascii', bytearray(b'\xff'), 0, 1, 'ouch'))
    raise AssertionError('expected TypeError')
except TypeError:
    pass

try:
    codecs.xmlcharrefreplace_errors(UnicodeTranslateError('あ', 0, 1, 'ouch'))
    raise AssertionError('expected TypeError')
except TypeError:
    pass
cs = (0, 1, 9, 10, 99, 100, 999, 1000, 9999, 10000, 99999, 100000, 999999, 1000000)
cs += (55296, 57343)
s = ''.join((chr(c) for c in cs))

assert codecs.xmlcharrefreplace_errors(UnicodeEncodeError('ascii', 'a' + s + 'b', 1, 1 + len(s), 'ouch')) == (''.join(('&#%d;' % c for c in cs)), 1 + len(s))
print("CodecCallbackTest::test_badandgoodxmlcharrefreplaceexceptions: ok")
