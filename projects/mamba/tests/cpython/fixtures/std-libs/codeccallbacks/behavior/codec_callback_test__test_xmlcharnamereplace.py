# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "codeccallbacks"
# dimension = "behavior"
# case = "codec_callback_test__test_xmlcharnamereplace"
# subject = "cpython.test_codeccallbacks.CodecCallbackTest.test_xmlcharnamereplace"
# kind = "semantic"
# xfail = "auto-ported CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_codeccallbacks.py"
# status = "filled"
# ///
# mamba-xfail: auto-ported CPython test; mamba promotion pending
# Auto-ported from CPython 3.12 test_codeccallbacks.py::CodecCallbackTest::test_xmlcharnamereplace
"""Auto-ported test: CodecCallbackTest::test_xmlcharnamereplace (CPython 3.12 oracle)."""


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
def xmlcharnamereplace(exc):
    if not isinstance(exc, UnicodeEncodeError):
        raise TypeError("don't know how to handle %r" % exc)
    l = []
    for c in exc.object[exc.start:exc.end]:
        try:
            l.append('&%s;' % html.entities.codepoint2name[ord(c)])
        except KeyError:
            l.append('&#%d;' % ord(c))
    return (''.join(l), exc.end)
codecs.register_error('test.xmlcharnamereplace', xmlcharnamereplace)
sin = '«ℜ» = 〈ሴ€〉'
sout = b'&laquo;&real;&raquo; = &lang;&#4660;&euro;&rang;'

assert sin.encode('ascii', 'test.xmlcharnamereplace') == sout
sout = b'\xab&real;\xbb = &lang;&#4660;&euro;&rang;'

assert sin.encode('latin-1', 'test.xmlcharnamereplace') == sout
sout = b'\xab&real;\xbb = &lang;&#4660;\xa4&rang;'

assert sin.encode('iso-8859-15', 'test.xmlcharnamereplace') == sout
print("CodecCallbackTest::test_xmlcharnamereplace: ok")
