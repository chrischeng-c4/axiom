# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "codeccallbacks"
# dimension = "behavior"
# case = "codec_callback_test__test_crashing_decode_handler"
# subject = "cpython.test_codeccallbacks.CodecCallbackTest.test_crashing_decode_handler"
# kind = "semantic"
# xfail = "auto-ported CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_codeccallbacks.py"
# status = "filled"
# ///
# mamba-xfail: auto-ported CPython test; mamba promotion pending
# Auto-ported from CPython 3.12 test_codeccallbacks.py::CodecCallbackTest::test_crashing_decode_handler
"""Auto-ported test: CodecCallbackTest::test_crashing_decode_handler (CPython 3.12 oracle)."""


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
def forward_shorter_than_end(exc):
    if isinstance(exc, UnicodeDecodeError):
        return ('�', exc.start + 1)
    else:
        raise TypeError("don't know how to handle %r" % exc)
codecs.register_error('test.forward_shorter_than_end', forward_shorter_than_end)

assert b'\xd8\xd8\xd8\xd8\xd8\x00\x00\x00'.decode('utf-16-le', 'test.forward_shorter_than_end') == '����Ø\x00'

assert b'\xd8\xd8\xd8\xd8\x00\xd8\x00\x00'.decode('utf-16-be', 'test.forward_shorter_than_end') == '����Ø\x00'

assert b'\x11\x11\x11\x11\x11\x00\x00\x00\x00\x00\x00'.decode('utf-32-le', 'test.forward_shorter_than_end') == '���ᄑ\x00'

assert b'\x11\x11\x11\x00\x00\x11\x11\x00\x00\x00\x00'.decode('utf-32-be', 'test.forward_shorter_than_end') == '���ᄑ\x00'

def replace_with_long(exc):
    if isinstance(exc, UnicodeDecodeError):
        exc.object = b'\x00' * 8
        return ('�', exc.start)
    else:
        raise TypeError("don't know how to handle %r" % exc)
codecs.register_error('test.replace_with_long', replace_with_long)

assert b'\x00'.decode('utf-16', 'test.replace_with_long') == '�\x00\x00\x00\x00'

assert b'\x00'.decode('utf-32', 'test.replace_with_long') == '�\x00\x00'
print("CodecCallbackTest::test_crashing_decode_handler: ok")
