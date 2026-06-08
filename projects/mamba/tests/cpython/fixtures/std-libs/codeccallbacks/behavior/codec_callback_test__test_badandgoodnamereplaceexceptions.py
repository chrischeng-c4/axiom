# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "codeccallbacks"
# dimension = "behavior"
# case = "codec_callback_test__test_badandgoodnamereplaceexceptions"
# subject = "cpython.test_codeccallbacks.CodecCallbackTest.test_badandgoodnamereplaceexceptions"
# kind = "semantic"
# xfail = "auto-ported CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_codeccallbacks.py"
# status = "filled"
# ///
# mamba-xfail: auto-ported CPython test; mamba promotion pending
# Auto-ported from CPython 3.12 test_codeccallbacks.py::CodecCallbackTest::test_badandgoodnamereplaceexceptions
"""Auto-ported test: CodecCallbackTest::test_badandgoodnamereplaceexceptions (CPython 3.12 oracle)."""


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

try:
    codecs.namereplace_errors(42)
    raise AssertionError('expected TypeError')
except TypeError:
    pass

try:
    codecs.namereplace_errors(UnicodeError('ouch'))
    raise AssertionError('expected TypeError')
except TypeError:
    pass

try:
    codecs.namereplace_errors(UnicodeDecodeError('ascii', bytearray(b'\xff'), 0, 1, 'ouch'))
    raise AssertionError('expected TypeError')
except TypeError:
    pass

try:
    codecs.namereplace_errors(UnicodeTranslateError('あ', 0, 1, 'ouch'))
    raise AssertionError('expected TypeError')
except TypeError:
    pass
tests = [('あ', '\\N{HIRAGANA LETTER A}'), ('\x00', '\\x00'), ('ﯹ', '\\N{ARABIC LIGATURE UIGHUR KIRGHIZ YEH WITH HAMZA ABOVE WITH ALEF MAKSURA ISOLATED FORM}'), ('\U000e007f', '\\N{CANCEL TAG}'), ('\U0010ffff', '\\U0010ffff'), ('\ud800', '\\ud800'), ('\udfff', '\\udfff'), ('\ud800\udfff', '\\ud800\\udfff')]
for s, r in tests:

    assert codecs.namereplace_errors(UnicodeEncodeError('ascii', 'a' + s + 'b', 1, 1 + len(s), 'ouch')) == (r, 1 + len(s))
print("CodecCallbackTest::test_badandgoodnamereplaceexceptions: ok")
