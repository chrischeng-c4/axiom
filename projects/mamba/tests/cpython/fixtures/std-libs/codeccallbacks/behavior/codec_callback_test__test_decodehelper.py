# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "codeccallbacks"
# dimension = "behavior"
# case = "codec_callback_test__test_decodehelper"
# subject = "cpython.test_codeccallbacks.CodecCallbackTest.test_decodehelper"
# kind = "semantic"
# xfail = "auto-ported CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_codeccallbacks.py"
# status = "filled"
# ///
# mamba-xfail: auto-ported CPython test; mamba promotion pending
# Auto-ported from CPython 3.12 test_codeccallbacks.py::CodecCallbackTest::test_decodehelper
"""Auto-ported test: CodecCallbackTest::test_decodehelper (CPython 3.12 oracle)."""


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
    b'\xff'.decode('ascii', 'test.unknown')
    raise AssertionError('expected LookupError')
except LookupError:
    pass

def baddecodereturn1(exc):
    return 42
codecs.register_error('test.baddecodereturn1', baddecodereturn1)

try:
    b'\xff'.decode('ascii', 'test.baddecodereturn1')
    raise AssertionError('expected TypeError')
except TypeError:
    pass

try:
    b'\\'.decode('unicode-escape', 'test.baddecodereturn1')
    raise AssertionError('expected TypeError')
except TypeError:
    pass

try:
    b'\\x0'.decode('unicode-escape', 'test.baddecodereturn1')
    raise AssertionError('expected TypeError')
except TypeError:
    pass

try:
    b'\\x0y'.decode('unicode-escape', 'test.baddecodereturn1')
    raise AssertionError('expected TypeError')
except TypeError:
    pass

try:
    b'\\Uffffeeee'.decode('unicode-escape', 'test.baddecodereturn1')
    raise AssertionError('expected TypeError')
except TypeError:
    pass

try:
    b'\\uyyyy'.decode('raw-unicode-escape', 'test.baddecodereturn1')
    raise AssertionError('expected TypeError')
except TypeError:
    pass

def baddecodereturn2(exc):
    return ('?', None)
codecs.register_error('test.baddecodereturn2', baddecodereturn2)

try:
    b'\xff'.decode('ascii', 'test.baddecodereturn2')
    raise AssertionError('expected TypeError')
except TypeError:
    pass
handler = PosReturn()
codecs.register_error('test.posreturn', handler.handle)
handler.pos = -1

assert b'\xff0'.decode('ascii', 'test.posreturn') == '<?>0'
handler.pos = -2

assert b'\xff0'.decode('ascii', 'test.posreturn') == '<?><?>'
handler.pos = -3

try:
    b'\xff0'.decode('ascii', 'test.posreturn')
    raise AssertionError('expected IndexError')
except IndexError:
    pass
handler.pos = 1

assert b'\xff0'.decode('ascii', 'test.posreturn') == '<?>0'
handler.pos = 2

assert b'\xff0'.decode('ascii', 'test.posreturn') == '<?>'
handler.pos = 3

try:
    b'\xff0'.decode('ascii', 'test.posreturn')
    raise AssertionError('expected IndexError')
except IndexError:
    pass
handler.pos = 6

assert b'\\uyyyy0'.decode('raw-unicode-escape', 'test.posreturn') == '<?>0'

class D(dict):

    def __getitem__(self, key):
        raise ValueError

try:
    codecs.charmap_decode(b'\xff', 'strict', {255: None})
    raise AssertionError('expected UnicodeError')
except UnicodeError:
    pass

try:
    codecs.charmap_decode(b'\xff', 'strict', D())
    raise AssertionError('expected ValueError')
except ValueError:
    pass

try:
    codecs.charmap_decode(b'\xff', 'strict', {255: sys.maxunicode + 1})
    raise AssertionError('expected TypeError')
except TypeError:
    pass
print("CodecCallbackTest::test_decodehelper: ok")
