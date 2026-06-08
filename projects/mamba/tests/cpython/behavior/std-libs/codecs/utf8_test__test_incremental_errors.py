# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "codecs"
# dimension = "behavior"
# case = "utf8_test__test_incremental_errors"
# subject = "cpython.test_codecs.UTF8Test.test_incremental_errors"
# kind = "semantic"
# xfail = "auto-ported CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_codecs.py"
# status = "filled"
# ///
# mamba-xfail: auto-ported CPython test; mamba promotion pending
# Auto-ported from CPython 3.12 test_codecs.py::UTF8Test::test_incremental_errors
"""Auto-ported test: UTF8Test::test_incremental_errors (CPython 3.12 oracle)."""


import codecs
import contextlib
import copy
import io
import pickle
import sys
import unittest
import encodings
from unittest import mock
from test import support
from test.support import os_helper


try:
    import _testcapi
except ImportError:
    _testcapi = None

try:
    import _testinternalcapi
except ImportError:
    _testinternalcapi = None

try:
    import ctypes
except ImportError:
    ctypes = None
    SIZEOF_WCHAR_T = -1
else:
    SIZEOF_WCHAR_T = ctypes.sizeof(ctypes.c_wchar)

def coding_checker(self, coder):

    def check(input, expect):
        self.assertEqual(coder(input), (expect, len(input)))
    return check

def is_code_page_present(cp):
    from ctypes import POINTER, WINFUNCTYPE, WinDLL
    from ctypes.wintypes import BOOL, BYTE, WCHAR, UINT, DWORD
    MAX_LEADBYTES = 12
    MAX_DEFAULTCHAR = 2
    MAX_PATH = 260

    class CPINFOEXW(ctypes.Structure):
        _fields_ = [('MaxCharSize', UINT), ('DefaultChar', BYTE * MAX_DEFAULTCHAR), ('LeadByte', BYTE * MAX_LEADBYTES), ('UnicodeDefaultChar', WCHAR), ('CodePage', UINT), ('CodePageName', WCHAR * MAX_PATH)]
    prototype = WINFUNCTYPE(BOOL, UINT, DWORD, POINTER(CPINFOEXW))
    GetCPInfoEx = prototype(('GetCPInfoExW', WinDLL('kernel32')))
    info = CPINFOEXW()
    return GetCPInfoEx(cp, 0, info)

class Queue(object):
    """
    queue: write bytes at one end, read bytes from the other end
    """

    def __init__(self, buffer):
        self._buffer = buffer

    def write(self, chars):
        self._buffer += chars

    def read(self, size=-1):
        if size < 0:
            s = self._buffer
            self._buffer = self._buffer[:0]
            return s
        else:
            s = self._buffer[:size]
            self._buffer = self._buffer[size:]
            return s

class MixInCheckStateHandling:

    def check_state_handling_decode(self, encoding, u, s):
        for i in range(len(s) + 1):
            d = codecs.getincrementaldecoder(encoding)()
            part1 = d.decode(s[:i])
            state = d.getstate()
            self.assertIsInstance(state[1], int)
            if not state[1]:
                d.setstate((state[0][:0], 0))
                self.assertTrue(not d.decode(state[0]))
                self.assertEqual(state, d.getstate())
            d = codecs.getincrementaldecoder(encoding)()
            d.setstate(state)
            part2 = d.decode(s[i:], True)
            self.assertEqual(u, part1 + part2)

    def check_state_handling_encode(self, encoding, u, s):
        for i in range(len(u) + 1):
            d = codecs.getincrementalencoder(encoding)()
            part1 = d.encode(u[:i])
            state = d.getstate()
            d = codecs.getincrementalencoder(encoding)()
            d.setstate(state)
            part2 = d.encode(u[i:], True)
            self.assertEqual(s, part1 + part2)

punycode_testcases = [('ليهمابتكلموشعربي؟', b'egbpdaj6bu4bxfgehfvwxn'), ('他们为什么不说中文', b'ihqwcrb4cv8a8dqg056pqjye'), ('他們爲什麽不說中文', b'ihqwctvzc91f659drss3x8bo0yb'), ('Pročprostěnemluvíčesky', b'Proprostnemluvesky-uyb24dma41a'), ('למההםפשוטלאמדבריםעברית', b'4dbcagdahymbxekheh6e0a7fei0b'), ('यहलोगहिन्दीक्योंनहींबोलसकतेहैं', b'i1baa7eci9glrd9b2ae1bj0hfcgg6iyaf8o0a1dig0cd'), ('なぜみんな日本語を話してくれないのか', b'n8jok5ay5dzabd5bym9f0cm5685rrjetr6pdxa'), ('세계의모든사람들이한국어를이해한다면얼마나좋을까', b'989aomsvi5e83db1d2a355cv1e0vak1dwrv93d5xbh15a0dt30a5jpsd879ccm6fea98c'), ('почемужеонинеговорятпорусски', b'b1abfaaepdrnnbgefbaDotcwatmq2g4l'), ('PorquénopuedensimplementehablarenEspañol', b'PorqunopuedensimplementehablarenEspaol-fmd56a'), ('TạisaohọkhôngthểchỉnóitiếngViệt', b'TisaohkhngthchnitingVit-kjcr8268qyxafd2f1b9g'), ('3年B組金八先生', b'3B-ww4c5e180e575a65lsy2b'), ('安室奈美恵-with-SUPER-MONKEYS', b'-with-SUPER-MONKEYS-pc58ag80a8qai00g7n9n'), ('Hello-Another-Way-それぞれの場所', b'Hello-Another-Way--fc4qua05auwb3674vfr0b'), ('ひとつ屋根の下2', b'2-u9tlzr9756bt3uc0v'), ('MajiでKoiする5秒前', b'MajiKoi5-783gue6qz075azm5e'), ('パフィーdeルンバ', b'de-jg4avhby1noc0d'), ('そのスピードで', b'd9juau41awczczp'), ('-> $1.00 <-', b'-> $1.00 <--')]

for i in punycode_testcases:
    if len(i) != 2:
        print(repr(i))

nameprep_tests = [(b'foo\xc2\xad\xcd\x8f\xe1\xa0\x86\xe1\xa0\x8bbar\xe2\x80\x8b\xe2\x81\xa0baz\xef\xb8\x80\xef\xb8\x88\xef\xb8\x8f\xef\xbb\xbf', b'foobarbaz'), (b'CAFE', b'cafe'), (b'\xc3\x9f', b'ss'), (b'\xc4\xb0', b'i\xcc\x87'), (b'\xc5\x83\xcd\xba', b'\xc5\x84 \xce\xb9'), (None, None), (b'j\xcc\x8c\xc2\xa0\xc2\xaa', b'\xc7\xb0 a'), (b'\xe1\xbe\xb7', b'\xe1\xbe\xb6\xce\xb9'), (b'\xc7\xb0', b'\xc7\xb0'), (b'\xce\x90', b'\xce\x90'), (b'\xce\xb0', b'\xce\xb0'), (b'\xe1\xba\x96', b'\xe1\xba\x96'), (b'\xe1\xbd\x96', b'\xe1\xbd\x96'), (b' ', b' '), (b'\xc2\xa0', b' '), (b'\xe1\x9a\x80', None), (b'\xe2\x80\x80', b' '), (b'\xe2\x80\x8b', b''), (b'\xe3\x80\x80', b' '), (b'\x10\x7f', b'\x10\x7f'), (b'\xc2\x85', None), (b'\xe1\xa0\x8e', None), (b'\xef\xbb\xbf', b''), (b'\xf0\x9d\x85\xb5', None), (b'\xef\x84\xa3', None), (b'\xf3\xb1\x88\xb4', None), (b'\xf4\x8f\x88\xb4', None), (b'\xf2\x8f\xbf\xbe', None), (b'\xf4\x8f\xbf\xbf', None), (b'\xed\xbd\x82', None), (b'\xef\xbf\xbd', None), (b'\xe2\xbf\xb5', None), (b'\xcd\x81', b'\xcc\x81'), (b'\xe2\x80\x8e', None), (b'\xe2\x80\xaa', None), (b'\xf3\xa0\x80\x81', None), (b'\xf3\xa0\x81\x82', None), (b'foo\xd6\xbebar', None), (b'foo\xef\xb5\x90bar', None), (b'foo\xef\xb9\xb6bar', b'foo \xd9\x8ebar'), (b'\xd8\xa71', None), (b'\xd8\xa71\xd8\xa8', b'\xd8\xa71\xd8\xa8'), (None, None), (b'X\xc2\xad\xc3\x9f\xc4\xb0\xe2\x84\xa1j\xcc\x8c\xc2\xa0\xc2\xaa\xce\xb0\xe2\x80\x80', b'xssi\xcc\x87tel\xc7\xb0 a\xce\xb0 '), (b'X\xc3\x9f\xe3\x8c\x96\xc4\xb0\xe2\x84\xa1\xe2\x92\x9f\xe3\x8c\x80', b'xss\xe3\x82\xad\xe3\x83\xad\xe3\x83\xa1\xe3\x83\xbc\xe3\x83\x88\xe3\x83\xabi\xcc\x87tel(d)\xe3\x82\xa2\xe3\x83\x91\xe3\x83\xbc\xe3\x83\x88')]

all_unicode_encodings = ['ascii', 'big5', 'big5hkscs', 'charmap', 'cp037', 'cp1006', 'cp1026', 'cp1125', 'cp1140', 'cp1250', 'cp1251', 'cp1252', 'cp1253', 'cp1254', 'cp1255', 'cp1256', 'cp1257', 'cp1258', 'cp424', 'cp437', 'cp500', 'cp720', 'cp737', 'cp775', 'cp850', 'cp852', 'cp855', 'cp856', 'cp857', 'cp858', 'cp860', 'cp861', 'cp862', 'cp863', 'cp864', 'cp865', 'cp866', 'cp869', 'cp874', 'cp875', 'cp932', 'cp949', 'cp950', 'euc_jis_2004', 'euc_jisx0213', 'euc_jp', 'euc_kr', 'gb18030', 'gb2312', 'gbk', 'hp_roman8', 'hz', 'idna', 'iso2022_jp', 'iso2022_jp_1', 'iso2022_jp_2', 'iso2022_jp_2004', 'iso2022_jp_3', 'iso2022_jp_ext', 'iso2022_kr', 'iso8859_1', 'iso8859_10', 'iso8859_11', 'iso8859_13', 'iso8859_14', 'iso8859_15', 'iso8859_16', 'iso8859_2', 'iso8859_3', 'iso8859_4', 'iso8859_5', 'iso8859_6', 'iso8859_7', 'iso8859_8', 'iso8859_9', 'johab', 'koi8_r', 'koi8_t', 'koi8_u', 'kz1048', 'latin_1', 'mac_cyrillic', 'mac_greek', 'mac_iceland', 'mac_latin2', 'mac_roman', 'mac_turkish', 'palmos', 'ptcp154', 'punycode', 'raw_unicode_escape', 'shift_jis', 'shift_jis_2004', 'shift_jisx0213', 'tis_620', 'unicode_escape', 'utf_16', 'utf_16_be', 'utf_16_le', 'utf_7', 'utf_8']

if hasattr(codecs, 'mbcs_encode'):
    all_unicode_encodings.append('mbcs')

if hasattr(codecs, 'oem_encode'):
    all_unicode_encodings.append('oem')

broken_unicode_with_stateful = ['punycode']

bytes_transform_encodings = ['base64_codec', 'uu_codec', 'quopri_codec', 'hex_codec']

transform_aliases = {'base64_codec': ['base64', 'base_64'], 'uu_codec': ['uu'], 'quopri_codec': ['quopri', 'quoted_printable', 'quotedprintable'], 'hex_codec': ['hex'], 'rot_13': ['rot13']}

try:
    import zlib
except ImportError:
    zlib = None
else:
    bytes_transform_encodings.append('zlib_codec')
    transform_aliases['zlib_codec'] = ['zip', 'zlib']

try:
    import bz2
except ImportError:
    pass
else:
    bytes_transform_encodings.append('bz2_codec')
    transform_aliases['bz2_codec'] = ['bz2']

_TEST_CODECS = {}

def _get_test_codec(codec_name):
    return _TEST_CODECS.get(codec_name)


# --- test body ---
ill_formed_sequence_replace = '�'
encoding = 'utf-8'
ill_formed_sequence = b'\xed\xb2\x80'
ill_formed_sequence_replace = '�' * 3
BOM = b''

def check_partial(input, partialresults):
    q = Queue(b'')
    r = codecs.getreader(encoding)(q)
    result = ''
    for c, partialresult in zip(input.encode(encoding), partialresults, strict=True):
        q.write(bytes([c]))
        result += r.read()

        assert result == partialresult

    assert r.read() == ''

    assert r.bytebuffer == b''
    d = codecs.getincrementaldecoder(encoding)()
    result = ''
    for c, partialresult in zip(input.encode(encoding), partialresults, strict=True):
        result += d.decode(bytes([c]))

        assert result == partialresult

    assert d.decode(b'', True) == ''

    assert d.buffer == b''
    d.reset()
    result = ''
    for c, partialresult in zip(input.encode(encoding), partialresults, strict=True):
        result += d.decode(bytes([c]))

        assert result == partialresult

    assert d.decode(b'', True) == ''

    assert d.buffer == b''
    encoded = input.encode(encoding)

    assert input == ''.join(codecs.iterdecode([bytes([c]) for c in encoded], encoding))

def check_state_handling_decode(encoding, u, s):
    for i in range(len(s) + 1):
        d = codecs.getincrementaldecoder(encoding)()
        part1 = d.decode(s[:i])
        state = d.getstate()

        assert isinstance(state[1], int)
        if not state[1]:
            d.setstate((state[0][:0], 0))

            assert not d.decode(state[0])

            assert state == d.getstate()
        d = codecs.getincrementaldecoder(encoding)()
        d.setstate(state)
        part2 = d.decode(s[i:], True)

        assert u == part1 + part2

def check_state_handling_encode(encoding, u, s):
    for i in range(len(u) + 1):
        d = codecs.getincrementalencoder(encoding)()
        part1 = d.encode(u[:i])
        state = d.getstate()
        d = codecs.getincrementalencoder(encoding)()
        d.setstate(state)
        part2 = d.encode(u[i:], True)

        assert s == part1 + part2
cases = [b'\x80', b'\xbf', b'\xc0', b'\xc1', b'\xf5', b'\xf6', b'\xff']
for prefix in (b'\xc2', b'\xdf', b'\xe0', b'\xe0\xa0', b'\xef', b'\xef\xbf', b'\xf0', b'\xf0\x90', b'\xf0\x90\x80', b'\xf4', b'\xf4\x8f', b'\xf4\x8f\xbf'):
    for suffix in (b'\x7f', b'\xc0'):
        cases.append(prefix + suffix)
cases.extend((b'\xe0\x80', b'\xe0\x9f', b'\xed\xa0\x80', b'\xed\xbf\xbf', b'\xf0\x80', b'\xf0\x8f', b'\xf4\x90'))
for data in cases:
    dec = codecs.getincrementaldecoder(encoding)()

    try:
        dec.decode(data)
        raise AssertionError('expected UnicodeDecodeError')
    except UnicodeDecodeError:
        pass
print("UTF8Test::test_incremental_errors: ok")
