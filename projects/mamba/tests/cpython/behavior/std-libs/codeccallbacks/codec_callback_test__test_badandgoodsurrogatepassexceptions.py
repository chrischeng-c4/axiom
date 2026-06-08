# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "codeccallbacks"
# dimension = "behavior"
# case = "codec_callback_test__test_badandgoodsurrogatepassexceptions"
# subject = "cpython.test_codeccallbacks.CodecCallbackTest.test_badandgoodsurrogatepassexceptions"
# kind = "semantic"
# xfail = "auto-ported CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_codeccallbacks.py"
# status = "filled"
# ///
# mamba-xfail: auto-ported CPython test; mamba promotion pending
# Auto-ported from CPython 3.12 test_codeccallbacks.py::CodecCallbackTest::test_badandgoodsurrogatepassexceptions
"""Auto-ported test: CodecCallbackTest::test_badandgoodsurrogatepassexceptions (CPython 3.12 oracle)."""


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
def check_exceptionobjectargs(exctype, args, msg):

    try:
        exctype(*args[:-1])
        raise AssertionError('expected TypeError')
    except TypeError:
        pass

    try:
        exctype(*args + ['too much'])
        raise AssertionError('expected TypeError')
    except TypeError:
        pass
    wrongargs = ['spam', b'eggs', b'spam', 42, 1.0, None]
    for i in range(len(args)):
        for wrongarg in wrongargs:
            if type(wrongarg) is type(args[i]):
                continue
            callargs = []
            for j in range(len(args)):
                if i == j:
                    callargs.append(wrongarg)
                else:
                    callargs.append(args[i])

            try:
                exctype(*callargs)
                raise AssertionError('expected TypeError')
            except TypeError:
                pass
    exc = exctype(*args)

    assert str(exc) == msg
surrogatepass_errors = codecs.lookup_error('surrogatepass')

try:
    surrogatepass_errors(42)
    raise AssertionError('expected TypeError')
except TypeError:
    pass

try:
    surrogatepass_errors(UnicodeError('ouch'))
    raise AssertionError('expected TypeError')
except TypeError:
    pass

try:
    surrogatepass_errors(UnicodeTranslateError('\ud800', 0, 1, 'ouch'))
    raise AssertionError('expected TypeError')
except TypeError:
    pass
for enc in ('utf-8', 'utf-16le', 'utf-16be', 'utf-32le', 'utf-32be'):

    try:
        surrogatepass_errors(UnicodeEncodeError(enc, 'a', 0, 1, 'ouch'))
        raise AssertionError('expected UnicodeEncodeError')
    except UnicodeEncodeError:
        pass

    try:
        surrogatepass_errors(UnicodeDecodeError(enc, 'a'.encode(enc), 0, 1, 'ouch'))
        raise AssertionError('expected UnicodeDecodeError')
    except UnicodeDecodeError:
        pass
for s in ('\ud800', '\udfff', '\ud800\udfff'):

    try:
        surrogatepass_errors(UnicodeEncodeError('ascii', s, 0, len(s), 'ouch'))
        raise AssertionError('expected UnicodeEncodeError')
    except UnicodeEncodeError:
        pass
tests = [('utf-8', '\ud800', b'\xed\xa0\x80', 3), ('utf-16le', '\ud800', b'\x00\xd8', 2), ('utf-16be', '\ud800', b'\xd8\x00', 2), ('utf-32le', '\ud800', b'\x00\xd8\x00\x00', 4), ('utf-32be', '\ud800', b'\x00\x00\xd8\x00', 4), ('utf-8', '\udfff', b'\xed\xbf\xbf', 3), ('utf-16le', '\udfff', b'\xff\xdf', 2), ('utf-16be', '\udfff', b'\xdf\xff', 2), ('utf-32le', '\udfff', b'\xff\xdf\x00\x00', 4), ('utf-32be', '\udfff', b'\x00\x00\xdf\xff', 4), ('utf-8', '\ud800\udfff', b'\xed\xa0\x80\xed\xbf\xbf', 3), ('utf-16le', '\ud800\udfff', b'\x00\xd8\xff\xdf', 2), ('utf-16be', '\ud800\udfff', b'\xd8\x00\xdf\xff', 2), ('utf-32le', '\ud800\udfff', b'\x00\xd8\x00\x00\xff\xdf\x00\x00', 4), ('utf-32be', '\ud800\udfff', b'\x00\x00\xd8\x00\x00\x00\xdf\xff', 4)]
for enc, s, b, n in tests:

    assert surrogatepass_errors(UnicodeEncodeError(enc, 'a' + s + 'b', 1, 1 + len(s), 'ouch')) == (b, 1 + len(s))

    assert surrogatepass_errors(UnicodeDecodeError(enc, bytearray(b'a' + b[:n] + b'b'), 1, 1 + n, 'ouch')) == (s[:1], 1 + n)
print("CodecCallbackTest::test_badandgoodsurrogatepassexceptions: ok")
