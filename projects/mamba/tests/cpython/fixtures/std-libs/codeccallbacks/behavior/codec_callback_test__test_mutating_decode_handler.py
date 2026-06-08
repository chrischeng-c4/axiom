# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "codeccallbacks"
# dimension = "behavior"
# case = "codec_callback_test__test_mutating_decode_handler"
# subject = "cpython.test_codeccallbacks.CodecCallbackTest.test_mutating_decode_handler"
# kind = "semantic"
# xfail = "auto-ported CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_codeccallbacks.py"
# status = "filled"
# ///
# mamba-xfail: auto-ported CPython test; mamba promotion pending
# Auto-ported from CPython 3.12 test_codeccallbacks.py::CodecCallbackTest::test_mutating_decode_handler
"""Auto-ported test: CodecCallbackTest::test_mutating_decode_handler (CPython 3.12 oracle)."""


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
baddata = [('ascii', b'\xff'), ('utf-7', b'++'), ('utf-8', b'\xff'), ('utf-16', b'\xff'), ('utf-32', b'\xff'), ('unicode-escape', b'\\u123g'), ('raw-unicode-escape', b'\\u123g')]

def replacing(exc):
    if isinstance(exc, UnicodeDecodeError):
        exc.object = 42
        return ('䉂', 0)
    else:
        raise TypeError("don't know how to handle %r" % exc)
codecs.register_error('test.replacing', replacing)
for encoding, data in baddata:
    try:
        data.decode(encoding, 'test.replacing')
        raise AssertionError('expected TypeError')
    except TypeError:
        pass

def mutating(exc):
    if isinstance(exc, UnicodeDecodeError):
        exc.object = b''
        return ('䉂', 0)
    else:
        raise TypeError("don't know how to handle %r" % exc)
codecs.register_error('test.mutating', mutating)
for encoding, data in baddata:

    assert data.decode(encoding, 'test.mutating') == '䉂'
print("CodecCallbackTest::test_mutating_decode_handler: ok")
