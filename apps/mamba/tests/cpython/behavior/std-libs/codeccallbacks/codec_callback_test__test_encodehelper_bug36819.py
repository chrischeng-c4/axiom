# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "codeccallbacks"
# dimension = "behavior"
# case = "codec_callback_test__test_encodehelper_bug36819"
# subject = "cpython.test_codeccallbacks.CodecCallbackTest.test_encodehelper_bug36819"
# kind = "semantic"
# xfail = "auto-ported CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_codeccallbacks.py"
# status = "filled"
# ///
# mamba-xfail: auto-ported CPython test; mamba promotion pending
# Auto-ported from CPython 3.12 test_codeccallbacks.py::CodecCallbackTest::test_encodehelper_bug36819
"""Auto-ported test: CodecCallbackTest::test_encodehelper_bug36819 (CPython 3.12 oracle)."""


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
handler = RepeatedPosReturn()
codecs.register_error('test.bug36819', handler.handle)
input = 'abcd\udc80'
encodings = ['ascii', 'latin1', 'utf-8', 'utf-16', 'utf-32']
encodings += ['iso-8859-15']
if sys.platform == 'win32':
    encodings = ['mbcs', 'oem']
handler.repl = '\udcff'
for enc in encodings:
    handler.count = 50
    try:
        input.encode(enc, 'test.bug36819')
        raise AssertionError('expected UnicodeEncodeError')
    except UnicodeEncodeError as _aR_e:
        import types as _types_aR
        cm = _types_aR.SimpleNamespace(exception=_aR_e)
    exc = cm.exception

    assert exc.start == 4

    assert exc.end == 5

    assert exc.object == input
if sys.platform == 'win32':
    handler.count = 50
    try:
        codecs.code_page_encode(437, input, 'test.bug36819')
        raise AssertionError('expected UnicodeEncodeError')
    except UnicodeEncodeError as _aR_e:
        import types as _types_aR
        cm = _types_aR.SimpleNamespace(exception=_aR_e)
    exc = cm.exception

    assert exc.start == 4

    assert exc.end == 5

    assert exc.object == input
handler.repl = 'x'
for enc in encodings:
    handler.count = 50
    encoded = input.encode(enc, 'test.bug36819')

    assert encoded.decode(enc) == 'abcdx' * 51
if sys.platform == 'win32':
    handler.count = 50
    encoded = codecs.code_page_encode(437, input, 'test.bug36819')

    assert encoded[0].decode() == 'abcdx' * 51

    assert encoded[1] == len(input)
print("CodecCallbackTest::test_encodehelper_bug36819: ok")
