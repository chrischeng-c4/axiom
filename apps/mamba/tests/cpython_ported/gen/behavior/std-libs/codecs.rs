use super::super::super::super::harness::*;

/// Ported from `tests/cpython/behavior/std-libs/codecs/ascii_test__test_decode.py`.
#[test]
fn test_gen_behavior_std_libs_codecs_ascii_test__test_decode() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "codecs"
# dimension = "behavior"
# case = "ascii_test__test_decode"
# subject = "cpython.test_codecs.ASCIITest.test_decode"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_codecs.py"
# status = "filled"
# ///
# Auto-ported from CPython 3.12 test_codecs.py::ASCIITest::test_decode
"""Auto-ported test: ASCIITest::test_decode (CPython 3.12 oracle)."""


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

assert b'abc'.decode('ascii') == 'abc'
print("ASCIITest::test_decode: ok")
"###);
    assert_output(&out, r###"ASCIITest::test_decode: ok
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/codecs/ascii_test__test_encode.py`.
#[test]
fn test_gen_behavior_std_libs_codecs_ascii_test__test_encode() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "codecs"
# dimension = "behavior"
# case = "ascii_test__test_encode"
# subject = "cpython.test_codecs.ASCIITest.test_encode"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_codecs.py"
# status = "filled"
# ///
# Auto-ported from CPython 3.12 test_codecs.py::ASCIITest::test_encode
"""Auto-ported test: ASCIITest::test_encode (CPython 3.12 oracle)."""


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

assert 'abc123'.encode('ascii') == b'abc123'
print("ASCIITest::test_encode: ok")
"###);
    assert_output(&out, r###"ASCIITest::test_encode: ok
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/codecs/ascii_test__test_encode_surrogateescape_error.py`.
#[test]
fn test_gen_behavior_std_libs_codecs_ascii_test__test_encode_surrogateescape_error() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "codecs"
# dimension = "behavior"
# case = "ascii_test__test_encode_surrogateescape_error"
# subject = "cpython.test_codecs.ASCIITest.test_encode_surrogateescape_error"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_codecs.py"
# status = "filled"
# ///
# Auto-ported from CPython 3.12 test_codecs.py::ASCIITest::test_encode_surrogateescape_error
"""Auto-ported test: ASCIITest::test_encode_surrogateescape_error (CPython 3.12 oracle)."""


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
try:
    '\udc80ÿ'.encode('ascii', 'surrogateescape')
    raise AssertionError('expected UnicodeEncodeError')
except UnicodeEncodeError:
    pass
print("ASCIITest::test_encode_surrogateescape_error: ok")
"###);
    assert_output(&out, r###"ASCIITest::test_encode_surrogateescape_error: ok
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/codecs/base64_codec_roundtrip.py`.
#[test]
fn test_gen_behavior_std_libs_codecs_base64_codec_roundtrip() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "codecs"
# dimension = "behavior"
# case = "base64_codec_roundtrip"
# subject = "codecs.encode"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_codecs.py"
# status = "filled"
# ///
"""codecs.encode: the base64_codec is a bytes->bytes transform reached via codecs.encode/decode: b'hello world' round-trips through base64_codec"""
import codecs

_data = b"hello world"
_b64 = codecs.encode(_data, "base64_codec")
assert isinstance(_b64, bytes), f"base64 codec returns bytes: {type(_b64)!r}"
_decoded = codecs.decode(_b64, "base64_codec")
assert _decoded == _data, f"base64 round-trip = {_decoded!r}"

print("base64_codec_roundtrip OK")
"###);
    assert_output(&out, r###"base64_codec_roundtrip OK
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/codecs/charmap_decode_dict_map.py`.
#[test]
fn test_gen_behavior_std_libs_codecs_charmap_decode_dict_map() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "codecs"
# dimension = "behavior"
# case = "charmap_decode_dict_map"
# subject = "codecs.charmap_decode"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_codecs.py"
# status = "filled"
# ///
"""codecs.charmap_decode: charmap_decode with an int->int dict map looks up each byte value: {0:ord('a'),1:ord('b'),2:ord('c')} yields ('abc',3) and a value of sys.maxunicode reaches the max code point"""
import codecs

import sys
_a, _b, _c = ord("a"), ord("b"), ord("c")
# int->int dict map.
assert codecs.charmap_decode(
    b"\x00\x01\x02", "strict", {0: _a, 1: _b, 2: _c}
) == ("abc", 3)
# Dict map may reach the maximum Unicode code point.
assert codecs.charmap_decode(
    b"\x00\x01\x02", "strict", {0: sys.maxunicode, 1: _b, 2: _c}
) == (chr(sys.maxunicode) + "bc", 3)

print("charmap_decode_dict_map OK")
"###);
    assert_output(&out, r###"charmap_decode_dict_map OK
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/codecs/charmap_decode_handlers_recover.py`.
#[test]
fn test_gen_behavior_std_libs_codecs_charmap_decode_handlers_recover() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "codecs"
# dimension = "behavior"
# case = "charmap_decode_handlers_recover"
# subject = "codecs.charmap_decode"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_codecs.py"
# status = "filled"
# ///
"""codecs.charmap_decode: on a missing map slot the error handlers recover: 'replace' yields the U+FFFD char, 'ignore' drops it, 'backslashreplace' emits the \\xHH escape"""
import codecs

# 'ab' has no slot for byte 0x02; handlers recover differently.
_short = "ab"
assert codecs.charmap_decode(b"\x00\x01\x02", "replace", _short) == ("ab�", 3)
assert codecs.charmap_decode(b"\x00\x01\x02", "ignore", _short) == ("ab", 3)
assert codecs.charmap_decode(
    b"\x00\x01\x02", "backslashreplace", _short
) == ("ab\\x02", 3)

print("charmap_decode_handlers_recover OK")
"###);
    assert_output(&out, r###"charmap_decode_handlers_recover OK
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/codecs/charmap_decode_string_map.py`.
#[test]
fn test_gen_behavior_std_libs_codecs_charmap_decode_string_map() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "codecs"
# dimension = "behavior"
# case = "charmap_decode_string_map"
# subject = "codecs.charmap_decode"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_codecs.py"
# status = "filled"
# ///
"""codecs.charmap_decode: charmap_decode with a string map indexes byte n -> map[n]: b'\\x00\\x01\\x02' over 'abc' yields ('abc', 3), including an astral target via '\\U0010ffffbc'"""
import codecs

# String map: byte n -> map[n].
assert codecs.charmap_decode(b"\x00\x01\x02", "strict", "abc") == ("abc", 3)
# String map may target astral code points.
assert codecs.charmap_decode(
    b"\x00\x01\x02", "strict", "\U0010ffffbc"
) == ("\U0010ffffbc", 3)

print("charmap_decode_string_map OK")
"###);
    assert_output(&out, r###"charmap_decode_string_map OK
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/codecs/charmap_test__test_decode_with_string_map.py`.
#[test]
fn test_gen_behavior_std_libs_codecs_charmap_test__test_decode_with_string_map() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "codecs"
# dimension = "behavior"
# case = "charmap_test__test_decode_with_string_map"
# subject = "cpython.test_codecs.CharmapTest.test_decode_with_string_map"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_codecs.py"
# status = "filled"
# ///
# Auto-ported from CPython 3.12 test_codecs.py::CharmapTest::test_decode_with_string_map
"""Auto-ported test: CharmapTest::test_decode_with_string_map (CPython 3.12 oracle)."""


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

assert codecs.charmap_decode(b'\x00\x01\x02', 'strict', 'abc') == ('abc', 3)

assert codecs.charmap_decode(b'\x00\x01\x02', 'strict', '\U0010ffffbc') == ('\U0010ffffbc', 3)

try:
    codecs.charmap_decode(b'\x00\x01\x02', 'strict', 'ab')
    raise AssertionError('expected UnicodeDecodeError')
except UnicodeDecodeError:
    pass

try:
    codecs.charmap_decode(b'\x00\x01\x02', 'strict', 'ab\ufffe')
    raise AssertionError('expected UnicodeDecodeError')
except UnicodeDecodeError:
    pass

assert codecs.charmap_decode(b'\x00\x01\x02', 'replace', 'ab') == ('ab�', 3)

assert codecs.charmap_decode(b'\x00\x01\x02', 'replace', 'ab\ufffe') == ('ab�', 3)

assert codecs.charmap_decode(b'\x00\x01\x02', 'backslashreplace', 'ab') == ('ab\\x02', 3)

assert codecs.charmap_decode(b'\x00\x01\x02', 'backslashreplace', 'ab\ufffe') == ('ab\\x02', 3)

assert codecs.charmap_decode(b'\x00\x01\x02', 'ignore', 'ab') == ('ab', 3)

assert codecs.charmap_decode(b'\x00\x01\x02', 'ignore', 'ab\ufffe') == ('ab', 3)
allbytes = bytes(range(256))

assert codecs.charmap_decode(allbytes, 'ignore', '') == ('', len(allbytes))
print("CharmapTest::test_decode_with_string_map: ok")
"###);
    assert_output(&out, r###"CharmapTest::test_decode_with_string_map: ok
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/codecs/codecs_module_test__test_lookup_issue1813.py`.
#[test]
fn test_gen_behavior_std_libs_codecs_codecs_module_test__test_lookup_issue1813() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "codecs"
# dimension = "behavior"
# case = "codecs_module_test__test_lookup_issue1813"
# subject = "cpython.test_codecs.CodecsModuleTest.test_lookup_issue1813"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_codecs.py"
# status = "filled"
# ///
# Auto-ported from CPython 3.12 test_codecs.py::CodecsModuleTest::test_lookup_issue1813
"""Auto-ported test: CodecsModuleTest::test_lookup_issue1813 (CPython 3.12 oracle)."""


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
c = codecs.lookup('ASCII')

assert c.name == 'ascii'
print("CodecsModuleTest::test_lookup_issue1813: ok")
"###);
    assert_output(&out, r###"CodecsModuleTest::test_lookup_issue1813: ok
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/codecs/codecs_module_test__test_undefined.py`.
#[test]
fn test_gen_behavior_std_libs_codecs_codecs_module_test__test_undefined() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "codecs"
# dimension = "behavior"
# case = "codecs_module_test__test_undefined"
# subject = "cpython.test_codecs.CodecsModuleTest.test_undefined"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_codecs.py"
# status = "filled"
# ///
# Auto-ported from CPython 3.12 test_codecs.py::CodecsModuleTest::test_undefined
"""Auto-ported test: CodecsModuleTest::test_undefined (CPython 3.12 oracle)."""


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

try:
    codecs.encode('abc', 'undefined')
    raise AssertionError('expected UnicodeError')
except UnicodeError:
    pass

try:
    codecs.decode(b'abc', 'undefined')
    raise AssertionError('expected UnicodeError')
except UnicodeError:
    pass

try:
    codecs.encode('', 'undefined')
    raise AssertionError('expected UnicodeError')
except UnicodeError:
    pass

try:
    codecs.decode(b'', 'undefined')
    raise AssertionError('expected UnicodeError')
except UnicodeError:
    pass
for errors in ('strict', 'ignore', 'replace', 'backslashreplace'):

    try:
        codecs.encode('abc', 'undefined', errors)
        raise AssertionError('expected UnicodeError')
    except UnicodeError:
        pass

    try:
        codecs.decode(b'abc', 'undefined', errors)
        raise AssertionError('expected UnicodeError')
    except UnicodeError:
        pass
print("CodecsModuleTest::test_undefined: ok")
"###);
    assert_output(&out, r###"CodecsModuleTest::test_undefined: ok
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/codecs/encoded_file_test__test_basic.py`.
#[test]
fn test_gen_behavior_std_libs_codecs_encoded_file_test__test_basic() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "codecs"
# dimension = "behavior"
# case = "encoded_file_test__test_basic"
# subject = "cpython.test_codecs.EncodedFileTest.test_basic"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_codecs.py"
# status = "filled"
# ///
# Auto-ported from CPython 3.12 test_codecs.py::EncodedFileTest::test_basic
"""Auto-ported test: EncodedFileTest::test_basic (CPython 3.12 oracle)."""


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
f = io.BytesIO(b'\xed\x95\x9c\n\xea\xb8\x80')
ef = codecs.EncodedFile(f, 'utf-16-le', 'utf-8')

assert ef.read() == b'\\\xd5\n\x00\x00\xae'
f = io.BytesIO()
ef = codecs.EncodedFile(f, 'utf-8', 'latin-1')
ef.write(b'\xc3\xbc')

assert f.getvalue() == b'\xfc'
print("EncodedFileTest::test_basic: ok")
"###);
    assert_output(&out, r###"EncodedFileTest::test_basic: ok
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/codecs/encodedfile_recodes_read_and_write.py`.
#[test]
fn test_gen_behavior_std_libs_codecs_encodedfile_recodes_read_and_write() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "codecs"
# dimension = "behavior"
# case = "encodedfile_recodes_read_and_write"
# subject = "codecs.EncodedFile"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_codecs.py"
# status = "filled"
# ///
"""codecs.EncodedFile: EncodedFile recodes between a file encoding and a data encoding: reading 'ü' (latin-1 on disk) as utf-8 and writing utf-8 'ü' lands latin-1 b'\\xfc' on disk; the wrapped file closes on context exit"""
import codecs

import io
# Reading recodes the file encoding to the data encoding; the base file closes.
_f = io.BytesIO(b"\xc3\xbc")  # 'ü' as utf-8
with codecs.EncodedFile(_f, "latin-1", "utf-8") as _ef:
    assert _ef.read() == b"\xfc", "utf-8 bytes recoded to latin-1"
assert _f.closed, "EncodedFile closes the wrapped file on exit"
# Writing recodes the data encoding to the file encoding.
_out = io.BytesIO()
_ef2 = codecs.EncodedFile(_out, "utf-8", "latin-1")
_ef2.write(b"\xc3\xbc")
assert _out.getvalue() == b"\xfc", f"recoded write = {_out.getvalue()!r}"

print("encodedfile_recodes_read_and_write OK")
"###);
    assert_output(&out, r###"encodedfile_recodes_read_and_write OK
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/codecs/escape_decode_handlers_recover.py`.
#[test]
fn test_gen_behavior_std_libs_codecs_escape_decode_handlers_recover() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "codecs"
# dimension = "behavior"
# case = "escape_decode_handlers_recover"
# subject = "codecs.escape_decode"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_codecs.py"
# status = "filled"
# ///
"""codecs.escape_decode: escape_decode error handlers recover from a truncated \\x and report consumed length: 'ignore' on b'[\\x]\\x' is (b'[]',6), 'replace' is (b'[?]?',6); a plain byte passes through (b'A0',2)"""
import codecs

_decode = codecs.escape_decode
# Error handlers recover from a truncated \x and report bytes consumed.
assert _decode(b"[\\x]\\x", "ignore") == (b"[]", 6)
assert _decode(b"[\\x]\\x", "replace") == (b"[?]?", 6)
assert _decode(b"[\\x0]\\x0", "ignore") == (b"[]", 8)
assert _decode(b"[\\x0]\\x0", "replace") == (b"[?]?", 8)
# A plain byte that is not a backslash passes through verbatim.
assert _decode(b"A0") == (b"A0", 2)

print("escape_decode_handlers_recover OK")
"###);
    assert_output(&out, r###"escape_decode_handlers_recover OK
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/codecs/escape_decode_test__test_errors.py`.
#[test]
fn test_gen_behavior_std_libs_codecs_escape_decode_test__test_errors() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "codecs"
# dimension = "behavior"
# case = "escape_decode_test__test_errors"
# subject = "cpython.test_codecs.EscapeDecodeTest.test_errors"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_codecs.py"
# status = "filled"
# ///
# Auto-ported from CPython 3.12 test_codecs.py::EscapeDecodeTest::test_errors
"""Auto-ported test: EscapeDecodeTest::test_errors (CPython 3.12 oracle)."""


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
decode = codecs.escape_decode

try:
    decode(b'\\x')
    raise AssertionError('expected ValueError')
except ValueError:
    pass

try:
    decode(b'[\\x]')
    raise AssertionError('expected ValueError')
except ValueError:
    pass

assert decode(b'[\\x]\\x', 'ignore') == (b'[]', 6)

assert decode(b'[\\x]\\x', 'replace') == (b'[?]?', 6)

try:
    decode(b'\\x0')
    raise AssertionError('expected ValueError')
except ValueError:
    pass

try:
    decode(b'[\\x0]')
    raise AssertionError('expected ValueError')
except ValueError:
    pass

assert decode(b'[\\x0]\\x0', 'ignore') == (b'[]', 8)

assert decode(b'[\\x0]\\x0', 'replace') == (b'[?]?', 8)
print("EscapeDecodeTest::test_errors: ok")
"###);
    assert_output(&out, r###"EscapeDecodeTest::test_errors: ok
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/codecs/escape_decode_test__test_raw.py`.
#[test]
fn test_gen_behavior_std_libs_codecs_escape_decode_test__test_raw() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "codecs"
# dimension = "behavior"
# case = "escape_decode_test__test_raw"
# subject = "cpython.test_codecs.EscapeDecodeTest.test_raw"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_codecs.py"
# status = "filled"
# ///
# Auto-ported from CPython 3.12 test_codecs.py::EscapeDecodeTest::test_raw
"""Auto-ported test: EscapeDecodeTest::test_raw (CPython 3.12 oracle)."""


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
decode = codecs.escape_decode
for b in range(256):
    b = bytes([b])
    if b != b'\\':

        assert decode(b + b'0') == (b + b'0', 2)
print("EscapeDecodeTest::test_raw: ok")
"###);
    assert_output(&out, r###"EscapeDecodeTest::test_raw: ok
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/codecs/hex_codec_roundtrip.py`.
#[test]
fn test_gen_behavior_std_libs_codecs_hex_codec_roundtrip() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "codecs"
# dimension = "behavior"
# case = "hex_codec_roundtrip"
# subject = "codecs.encode"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_codecs.py"
# status = "filled"
# ///
"""codecs.encode: the hex_codec maps bytes to lowercase hex: b'\\xde\\xad\\xbe\\xef' encodes to b'deadbeef' and decodes back"""
import codecs

_data = b"\xde\xad\xbe\xef"
_hex = codecs.encode(_data, "hex_codec")
assert _hex == b"deadbeef", f"hex_codec = {_hex!r}"
_back = codecs.decode(_hex, "hex_codec")
assert _back == _data, f"hex decode = {_back!r}"

print("hex_codec_roundtrip OK")
"###);
    assert_output(&out, r###"hex_codec_roundtrip OK
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/codecs/idna_codec_test__test_errors.py`.
#[test]
fn test_gen_behavior_std_libs_codecs_idna_codec_test__test_errors() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "codecs"
# dimension = "behavior"
# case = "idna_codec_test__test_errors"
# subject = "cpython.test_codecs.IDNACodecTest.test_errors"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_codecs.py"
# status = "filled"
# ///
# Auto-ported from CPython 3.12 test_codecs.py::IDNACodecTest::test_errors
"""Auto-ported test: IDNACodecTest::test_errors (CPython 3.12 oracle)."""


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
"""Only supports "strict" error handler"""
'python.org'.encode('idna', 'strict')
b'python.org'.decode('idna', 'strict')
for errors in ('ignore', 'replace', 'backslashreplace', 'surrogateescape'):

    try:
        'python.org'.encode('idna', errors)
        raise AssertionError('expected Exception')
    except Exception:
        pass

    try:
        b'python.org'.decode('idna', errors)
        raise AssertionError('expected Exception')
    except Exception:
        pass
print("IDNACodecTest::test_errors: ok")
"###);
    assert_output(&out, r###"IDNACodecTest::test_errors: ok
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/codecs/idna_encode_decode.py`.
#[test]
fn test_gen_behavior_std_libs_codecs_idna_encode_decode() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "codecs"
# dimension = "behavior"
# case = "idna_encode_decode"
# subject = "str.encode"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_codecs.py"
# status = "filled"
# ///
"""str.encode: idna applies ToASCII/ToUnicode: 'pythön.org'.encode('idna') is b'xn--pythn-mua.org' and decodes back, while pure-ASCII labels pass through unchanged"""
import codecs

# ASCII labels pass through; non-ASCII labels get the xn-- prefix.
assert "python.org".encode("idna") == b"python.org"
assert "python.org.".encode("idna") == b"python.org."
assert "pythön.org".encode("idna") == b"xn--pythn-mua.org"
assert "pythön.org.".encode("idna") == b"xn--pythn-mua.org."
# decode is the inverse of encode.
assert b"xn--pythn-mua.org".decode("idna") == "pythön.org"
assert b"python.org".decode("idna") == "python.org"

print("idna_encode_decode OK")
"###);
    assert_output(&out, r###"idna_encode_decode OK
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/codecs/iterencode_iterdecode_stream_chunks.py`.
#[test]
fn test_gen_behavior_std_libs_codecs_iterencode_iterdecode_stream_chunks() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "codecs"
# dimension = "behavior"
# case = "iterencode_iterdecode_stream_chunks"
# subject = "codecs.iterencode"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_codecs.py"
# status = "filled"
# ///
"""codecs.iterencode: iterencode joins a chunked sequence into the full byte string and iterdecode is its inverse: ['foo','bar','café'] -> 'foobarcafé'.encode('utf-8') and back"""
import codecs

_joined = b"".join(codecs.iterencode(["foo", "bar", "café"], "utf-8"))
assert _joined == "foobarcafé".encode("utf-8"), f"iterencode = {_joined!r}"
_chunks = ["foo".encode("utf-8"), "café".encode("utf-8")]
_text = "".join(codecs.iterdecode(_chunks, "utf-8"))
assert _text == "foocafé", f"iterdecode = {_text!r}"

print("iterencode_iterdecode_stream_chunks OK")
"###);
    assert_output(&out, r###"iterencode_iterdecode_stream_chunks OK
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/codecs/lookup_normalizes_names.py`.
#[test]
fn test_gen_behavior_std_libs_codecs_lookup_normalizes_names() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "codecs"
# dimension = "behavior"
# case = "lookup_normalizes_names"
# subject = "codecs.lookup"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_codecs.py"
# status = "filled"
# ///
"""codecs.lookup: codecs.lookup normalizes codec names: lookup('utf-8'), lookup('UTF-8'), lookup('utf_8') all report .name == 'utf-8'"""
import codecs

_a = codecs.lookup("utf-8")
_b = codecs.lookup("UTF-8")
_c = codecs.lookup("utf_8")
assert _a.name == _b.name == _c.name == "utf-8", \
    f"normalized names: {_a.name!r} {_b.name!r} {_c.name!r}"

print("lookup_normalizes_names OK")
"###);
    assert_output(&out, r###"lookup_normalizes_names OK
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/codecs/normalize_encoding_folds_separators.py`.
#[test]
fn test_gen_behavior_std_libs_codecs_normalize_encoding_folds_separators() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "codecs"
# dimension = "behavior"
# case = "normalize_encoding_folds_separators"
# subject = "encodings.normalize_encoding"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_codecs.py"
# status = "filled"
# ///
"""encodings.normalize_encoding: encodings.normalize_encoding folds whitespace runs to a single underscore while preserving case and dots: 'utf   8' -> 'utf_8', 'UTF 8' -> 'UTF_8', 'utf.8' -> 'utf.8'"""
import encodings

_normalize = encodings.normalize_encoding
assert _normalize("utf_8") == "utf_8"
assert _normalize("utf   8") == "utf_8"
assert _normalize("UTF 8") == "UTF_8"  # case is preserved
assert _normalize("utf.8") == "utf.8"  # dots are kept

print("normalize_encoding_folds_separators OK")
"###);
    assert_output(&out, r###"normalize_encoding_folds_separators OK
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/codecs/punycode_roundtrip.py`.
#[test]
fn test_gen_behavior_std_libs_codecs_punycode_roundtrip() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "codecs"
# dimension = "behavior"
# case = "punycode_roundtrip"
# subject = "str.encode"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_codecs.py"
# status = "filled"
# ///
"""str.encode: punycode bootstring round-trips a non-ASCII string ('他们为什么不说中文' -> b'ihqwcrb4cv8a8dqg056pqjye') and appends only '-' for pure ASCII ('abc' -> b'abc-')"""
import codecs

_uni = "他们为什么不说中文"
_puny = _uni.encode("punycode")
assert _puny == b"ihqwcrb4cv8a8dqg056pqjye", f"punycode encode = {_puny!r}"
assert _puny.decode("punycode") == _uni, "punycode round-trip"
# Pure-ASCII just appends the '-' delimiter.
assert "abc".encode("punycode") == b"abc-"

print("punycode_roundtrip OK")
"###);
    assert_output(&out, r###"punycode_roundtrip OK
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/codecs/raw_unicode_escape_decode.py`.
#[test]
fn test_gen_behavior_std_libs_codecs_raw_unicode_escape_decode() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "codecs"
# dimension = "behavior"
# case = "raw_unicode_escape_decode"
# subject = "codecs.raw_unicode_escape_decode"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_codecs.py"
# status = "filled"
# ///
"""codecs.raw_unicode_escape_decode: raw_unicode_escape_decode keeps \\xHH literal but decodes \\u escapes: b'\\\\xff' -> ('\\\\xff',4) and b'\\\\u00e9' -> ('é',6)"""
import codecs

# raw_unicode_escape keeps \xHH literal but decodes \u escapes.
assert codecs.raw_unicode_escape_decode(b"\\xff") == ("\\xff", 4)
assert codecs.raw_unicode_escape_decode(b"\\u00e9") == ("é", 6)

print("raw_unicode_escape_decode OK")
"###);
    assert_output(&out, r###"raw_unicode_escape_decode OK
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/codecs/raw_unicode_escape_test__test_empty.py`.
#[test]
fn test_gen_behavior_std_libs_codecs_raw_unicode_escape_test__test_empty() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "codecs"
# dimension = "behavior"
# case = "raw_unicode_escape_test__test_empty"
# subject = "cpython.test_codecs.RawUnicodeEscapeTest.test_empty"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_codecs.py"
# status = "filled"
# ///
# Auto-ported from CPython 3.12 test_codecs.py::RawUnicodeEscapeTest::test_empty
"""Auto-ported test: RawUnicodeEscapeTest::test_empty (CPython 3.12 oracle)."""


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
encoding = 'raw-unicode-escape'
test_lone_surrogates = None

assert codecs.raw_unicode_escape_encode('') == (b'', 0)

assert codecs.raw_unicode_escape_decode(b'') == ('', 0)
print("RawUnicodeEscapeTest::test_empty: ok")
"###);
    assert_output(&out, r###"RawUnicodeEscapeTest::test_empty: ok
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/codecs/raw_unicode_escape_test__test_raw_decode.py`.
#[test]
fn test_gen_behavior_std_libs_codecs_raw_unicode_escape_test__test_raw_decode() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "codecs"
# dimension = "behavior"
# case = "raw_unicode_escape_test__test_raw_decode"
# subject = "cpython.test_codecs.RawUnicodeEscapeTest.test_raw_decode"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_codecs.py"
# status = "filled"
# ///
# Auto-ported from CPython 3.12 test_codecs.py::RawUnicodeEscapeTest::test_raw_decode
"""Auto-ported test: RawUnicodeEscapeTest::test_raw_decode (CPython 3.12 oracle)."""


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
encoding = 'raw-unicode-escape'
test_lone_surrogates = None
decode = codecs.raw_unicode_escape_decode
for b in range(256):

    assert decode(bytes([b]) + b'0') == (chr(b) + '0', 2)
print("RawUnicodeEscapeTest::test_raw_decode: ok")
"###);
    assert_output(&out, r###"RawUnicodeEscapeTest::test_raw_decode: ok
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/codecs/raw_unicode_escape_test__test_raw_encode.py`.
#[test]
fn test_gen_behavior_std_libs_codecs_raw_unicode_escape_test__test_raw_encode() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "codecs"
# dimension = "behavior"
# case = "raw_unicode_escape_test__test_raw_encode"
# subject = "cpython.test_codecs.RawUnicodeEscapeTest.test_raw_encode"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_codecs.py"
# status = "filled"
# ///
# Auto-ported from CPython 3.12 test_codecs.py::RawUnicodeEscapeTest::test_raw_encode
"""Auto-ported test: RawUnicodeEscapeTest::test_raw_encode (CPython 3.12 oracle)."""


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
encoding = 'raw-unicode-escape'
test_lone_surrogates = None
encode = codecs.raw_unicode_escape_encode
for b in range(256):

    assert encode(chr(b)) == (bytes([b]), 1)
print("RawUnicodeEscapeTest::test_raw_encode: ok")
"###);
    assert_output(&out, r###"RawUnicodeEscapeTest::test_raw_encode: ok
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/codecs/read_buffer_test__test_bad_args.py`.
#[test]
fn test_gen_behavior_std_libs_codecs_read_buffer_test__test_bad_args() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "codecs"
# dimension = "behavior"
# case = "read_buffer_test__test_bad_args"
# subject = "cpython.test_codecs.ReadBufferTest.test_bad_args"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_codecs.py"
# status = "filled"
# ///
# Auto-ported from CPython 3.12 test_codecs.py::ReadBufferTest::test_bad_args
"""Auto-ported test: ReadBufferTest::test_bad_args (CPython 3.12 oracle)."""


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

try:
    codecs.readbuffer_encode()
    raise AssertionError('expected TypeError')
except TypeError:
    pass

try:
    codecs.readbuffer_encode(42)
    raise AssertionError('expected TypeError')
except TypeError:
    pass
print("ReadBufferTest::test_bad_args: ok")
"###);
    assert_output(&out, r###"ReadBufferTest::test_bad_args: ok
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/codecs/read_buffer_test__test_empty.py`.
#[test]
fn test_gen_behavior_std_libs_codecs_read_buffer_test__test_empty() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "codecs"
# dimension = "behavior"
# case = "read_buffer_test__test_empty"
# subject = "cpython.test_codecs.ReadBufferTest.test_empty"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_codecs.py"
# status = "filled"
# ///
# Auto-ported from CPython 3.12 test_codecs.py::ReadBufferTest::test_empty
"""Auto-ported test: ReadBufferTest::test_empty (CPython 3.12 oracle)."""


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

assert codecs.readbuffer_encode('') == (b'', 0)
print("ReadBufferTest::test_empty: ok")
"###);
    assert_output(&out, r###"ReadBufferTest::test_empty: ok
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/codecs/readbuffer_encode_accepts_buffer.py`.
#[test]
fn test_gen_behavior_std_libs_codecs_readbuffer_encode_accepts_buffer() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "codecs"
# dimension = "behavior"
# case = "readbuffer_encode_accepts_buffer"
# subject = "codecs.readbuffer_encode"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_codecs.py"
# status = "filled"
# ///
"""codecs.readbuffer_encode: readbuffer_encode accepts any buffer (e.g. a bytearray) and returns (bytes, length): readbuffer_encode(bytearray(b'spam')) is (b'spam', 4)"""
import codecs

assert codecs.readbuffer_encode(bytearray(b"spam")) == (b"spam", 4)

print("readbuffer_encode_accepts_buffer OK")
"###);
    assert_output(&out, r###"readbuffer_encode_accepts_buffer OK
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/codecs/stream_recoder_test__test_seeking_read.py`.
#[test]
fn test_gen_behavior_std_libs_codecs_stream_recoder_test__test_seeking_read() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "codecs"
# dimension = "behavior"
# case = "stream_recoder_test__test_seeking_read"
# subject = "cpython.test_codecs.StreamRecoderTest.test_seeking_read"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_codecs.py"
# status = "filled"
# ///
# Auto-ported from CPython 3.12 test_codecs.py::StreamRecoderTest::test_seeking_read
"""Auto-ported test: StreamRecoderTest::test_seeking_read (CPython 3.12 oracle)."""


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
bio = io.BytesIO('line1\nline2\nline3\n'.encode('utf-16-le'))
sr = codecs.EncodedFile(bio, 'utf-8', 'utf-16-le')

assert sr.readline() == b'line1\n'
sr.seek(0)

assert sr.readline() == b'line1\n'

assert sr.readline() == b'line2\n'

assert sr.readline() == b'line3\n'

assert sr.readline() == b''
print("StreamRecoderTest::test_seeking_read: ok")
"###);
    assert_output(&out, r###"StreamRecoderTest::test_seeking_read: ok
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/codecs/streamrecoder_readline_seek.py`.
#[test]
fn test_gen_behavior_std_libs_codecs_streamrecoder_readline_seek() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "codecs"
# dimension = "behavior"
# case = "streamrecoder_readline_seek"
# subject = "codecs.EncodedFile"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_codecs.py"
# status = "filled"
# ///
"""codecs.EncodedFile: a StreamRecoder (via EncodedFile over utf-16-le) supports readline and seek(0): two utf-16-le lines read back as b'line1\\n'/b'line2\\n' and seek(0) rewinds to the first"""
import codecs

import io
_bio = io.BytesIO("line1\nline2\n".encode("utf-16-le"))
_sr = codecs.EncodedFile(_bio, "utf-8", "utf-16-le")
assert _sr.readline() == b"line1\n"
_sr.seek(0)
assert _sr.readline() == b"line1\n", "seek(0) rewinds StreamRecoder"
assert _sr.readline() == b"line2\n"

print("streamrecoder_readline_seek OK")
"###);
    assert_output(&out, r###"streamrecoder_readline_seek OK
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/codecs/surrogateescape_roundtrip.py`.
#[test]
fn test_gen_behavior_std_libs_codecs_surrogateescape_roundtrip() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "codecs"
# dimension = "behavior"
# case = "surrogateescape_roundtrip"
# subject = "str.encode"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_codecs.py"
# status = "filled"
# ///
"""str.encode: surrogateescape maps undecodable bytes 0x80-0xFF to lone surrogates U+DC80-U+DCFF and back, for both ascii and utf-8, including an ill-formed 3-byte sequence"""
import codecs

# ascii smuggles a high byte through a lone surrogate.
assert b"foo\x80bar".decode("ascii", "surrogateescape") == "foo\udc80bar"
assert "foo\udc80bar".encode("ascii", "surrogateescape") == b"foo\x80bar"
# utf-8 round-trips the same way.
assert b"foo\x80bar".decode("utf-8", "surrogateescape") == "foo\udc80bar"
assert "foo\udc80bar".encode("utf-8", "surrogateescape") == b"foo\x80bar"
# An ill-formed 3-byte UTF-8 surrogate becomes three escapes.
assert b"\xed\xb0\x80".decode("utf-8", "surrogateescape") == "\udced\udcb0\udc80"
assert "\udced\udcb0\udc80".encode("utf-8", "surrogateescape") == b"\xed\xb0\x80"

print("surrogateescape_roundtrip OK")
"###);
    assert_output(&out, r###"surrogateescape_roundtrip OK
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/codecs/surrogatepass_roundtrip.py`.
#[test]
fn test_gen_behavior_std_libs_codecs_surrogatepass_roundtrip() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "codecs"
# dimension = "behavior"
# case = "surrogatepass_roundtrip"
# subject = "str.encode"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_codecs.py"
# status = "filled"
# ///
"""str.encode: surrogatepass lets a lone surrogate pass through utf-8: '\\ud901'.encode('utf-8','surrogatepass') is b'\\xed\\xa4\\x81' and decodes back to '\\ud901'"""
import codecs

_data = "\ud901".encode("utf-8", "surrogatepass")
assert _data == b"\xed\xa4\x81", f"surrogatepass utf-8 = {_data!r}"
assert _data.decode("utf-8", "surrogatepass") == "\ud901"

print("surrogatepass_roundtrip OK")
"###);
    assert_output(&out, r###"surrogatepass_roundtrip OK
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/codecs/undefined_codec_refuses_everything.py`.
#[test]
fn test_gen_behavior_std_libs_codecs_undefined_codec_refuses_everything() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "codecs"
# dimension = "behavior"
# case = "undefined_codec_refuses_everything"
# subject = "codecs.encode"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_codecs.py"
# status = "filled"
# ///
"""codecs.encode: the 'undefined' codec raises UnicodeError for encode and decode of any input, and even non-strict handlers (ignore/replace/backslashreplace) cannot make it succeed"""
import codecs

# The 'undefined' codec refuses to encode or decode anything.
for _call in (
    lambda: codecs.encode("abc", "undefined"),
    lambda: codecs.decode(b"abc", "undefined"),
    lambda: codecs.encode("", "undefined"),
):
    _raised = False
    try:
        _call()
    except UnicodeError:
        _raised = True
    assert _raised, "'undefined' codec should raise UnicodeError"
# Even non-strict handlers cannot make 'undefined' succeed.
for _errors in ("strict", "ignore", "replace", "backslashreplace"):
    _raised = False
    try:
        codecs.encode("abc", "undefined", _errors)
    except UnicodeError:
        _raised = True
    assert _raised, f"'undefined' with {_errors!r} still raises"

print("undefined_codec_refuses_everything OK")
"###);
    assert_output(&out, r###"undefined_codec_refuses_everything OK
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/codecs/unicode_escape_decode_handlers_recover.py`.
#[test]
fn test_gen_behavior_std_libs_codecs_unicode_escape_decode_handlers_recover() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "codecs"
# dimension = "behavior"
# case = "unicode_escape_decode_handlers_recover"
# subject = "codecs.unicode_escape_decode"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_codecs.py"
# status = "filled"
# ///
"""codecs.unicode_escape_decode: unicode_escape_decode handlers recover from truncated \\x/\\u/\\U and from a code point past U+10FFFF; unicode_escape_encode leaves printable ASCII as-is"""
import codecs

_udecode = codecs.unicode_escape_decode
# Truncated \x \u \U: handlers recover and report consumed length.
for _c in (b"x", b"u", b"U"):
    _data = b"[\\" + _c + b"0]\\" + _c + b"0"
    assert _udecode(_data, "ignore") == ("[]", len(_data))
    assert _udecode(_data, "replace") == ("[�]�", len(_data))
# A code point past U+10FFFF is an error; handlers recover.
assert _udecode(b"\\U00110000", "ignore") == ("", 10)
assert _udecode(b"\\U00110000", "replace") == ("�", 10)
# unicode_escape_encode leaves printable ASCII as-is.
assert codecs.unicode_escape_encode("A") == (b"A", 1)

print("unicode_escape_decode_handlers_recover OK")
"###);
    assert_output(&out, r###"unicode_escape_decode_handlers_recover OK
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/codecs/unicode_escape_test__test_decode_errors.py`.
#[test]
fn test_gen_behavior_std_libs_codecs_unicode_escape_test__test_decode_errors() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "codecs"
# dimension = "behavior"
# case = "unicode_escape_test__test_decode_errors"
# subject = "cpython.test_codecs.UnicodeEscapeTest.test_decode_errors"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_codecs.py"
# status = "filled"
# ///
# Auto-ported from CPython 3.12 test_codecs.py::UnicodeEscapeTest::test_decode_errors
"""Auto-ported test: UnicodeEscapeTest::test_decode_errors (CPython 3.12 oracle)."""


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
encoding = 'unicode-escape'
test_lone_surrogates = None
decode = codecs.unicode_escape_decode
for c, d in ((b'x', 2), (b'u', 4), (b'U', 4)):
    for i in range(d):

        try:
            decode(b'\\' + c + b'0' * i)
            raise AssertionError('expected UnicodeDecodeError')
        except UnicodeDecodeError:
            pass

        try:
            decode(b'[\\' + c + b'0' * i + b']')
            raise AssertionError('expected UnicodeDecodeError')
        except UnicodeDecodeError:
            pass
        data = b'[\\' + c + b'0' * i + b']\\' + c + b'0' * i

        assert decode(data, 'ignore') == ('[]', len(data))

        assert decode(data, 'replace') == ('[�]�', len(data))

try:
    decode(b'\\U00110000')
    raise AssertionError('expected UnicodeDecodeError')
except UnicodeDecodeError:
    pass

assert decode(b'\\U00110000', 'ignore') == ('', 10)

assert decode(b'\\U00110000', 'replace') == ('�', 10)
print("UnicodeEscapeTest::test_decode_errors: ok")
"###);
    assert_output(&out, r###"UnicodeEscapeTest::test_decode_errors: ok
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/codecs/unicode_escape_test__test_empty.py`.
#[test]
fn test_gen_behavior_std_libs_codecs_unicode_escape_test__test_empty() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "codecs"
# dimension = "behavior"
# case = "unicode_escape_test__test_empty"
# subject = "cpython.test_codecs.UnicodeEscapeTest.test_empty"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_codecs.py"
# status = "filled"
# ///
# Auto-ported from CPython 3.12 test_codecs.py::UnicodeEscapeTest::test_empty
"""Auto-ported test: UnicodeEscapeTest::test_empty (CPython 3.12 oracle)."""


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
encoding = 'unicode-escape'
test_lone_surrogates = None

assert codecs.unicode_escape_encode('') == (b'', 0)

assert codecs.unicode_escape_decode(b'') == ('', 0)
print("UnicodeEscapeTest::test_empty: ok")
"###);
    assert_output(&out, r###"UnicodeEscapeTest::test_empty: ok
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/codecs/unicode_escape_test__test_raw_decode.py`.
#[test]
fn test_gen_behavior_std_libs_codecs_unicode_escape_test__test_raw_decode() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "codecs"
# dimension = "behavior"
# case = "unicode_escape_test__test_raw_decode"
# subject = "cpython.test_codecs.UnicodeEscapeTest.test_raw_decode"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_codecs.py"
# status = "filled"
# ///
# Auto-ported from CPython 3.12 test_codecs.py::UnicodeEscapeTest::test_raw_decode
"""Auto-ported test: UnicodeEscapeTest::test_raw_decode (CPython 3.12 oracle)."""


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
encoding = 'unicode-escape'
test_lone_surrogates = None
decode = codecs.unicode_escape_decode
for b in range(256):
    if b != b'\\'[0]:

        assert decode(bytes([b]) + b'0') == (chr(b) + '0', 2)
print("UnicodeEscapeTest::test_raw_decode: ok")
"###);
    assert_output(&out, r###"UnicodeEscapeTest::test_raw_decode: ok
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/codecs/unicode_escape_test__test_raw_encode.py`.
#[test]
fn test_gen_behavior_std_libs_codecs_unicode_escape_test__test_raw_encode() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "codecs"
# dimension = "behavior"
# case = "unicode_escape_test__test_raw_encode"
# subject = "cpython.test_codecs.UnicodeEscapeTest.test_raw_encode"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_codecs.py"
# status = "filled"
# ///
# Auto-ported from CPython 3.12 test_codecs.py::UnicodeEscapeTest::test_raw_encode
"""Auto-ported test: UnicodeEscapeTest::test_raw_encode (CPython 3.12 oracle)."""


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
encoding = 'unicode-escape'
test_lone_surrogates = None
encode = codecs.unicode_escape_encode
for b in range(32, 127):
    if b != b'\\'[0]:

        assert encode(chr(b)) == (bytes([b]), 1)
print("UnicodeEscapeTest::test_raw_encode: ok")
"###);
    assert_output(&out, r###"UnicodeEscapeTest::test_raw_encode: ok
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/codecs/utf16_be_test__test_nonbmp.py`.
#[test]
fn test_gen_behavior_std_libs_codecs_utf16_be_test__test_nonbmp() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "codecs"
# dimension = "behavior"
# case = "utf16_be_test__test_nonbmp"
# subject = "cpython.test_codecs.UTF16BETest.test_nonbmp"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_codecs.py"
# status = "filled"
# ///
# Auto-ported from CPython 3.12 test_codecs.py::UTF16BETest::test_nonbmp
"""Auto-ported test: UTF16BETest::test_nonbmp (CPython 3.12 oracle)."""


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
encoding = 'utf-16-be'
ill_formed_sequence = b'\xdc\x80'

assert '\U00010203'.encode(encoding) == b'\xd8\x00\xde\x03'

assert b'\xd8\x00\xde\x03'.decode(encoding) == '\U00010203'
print("UTF16BETest::test_nonbmp: ok")
"###);
    assert_output(&out, r###"UTF16BETest::test_nonbmp: ok
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/codecs/utf16_decode_handlers_on_lone_byte.py`.
#[test]
fn test_gen_behavior_std_libs_codecs_utf16_decode_handlers_on_lone_byte() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "codecs"
# dimension = "behavior"
# case = "utf16_decode_handlers_on_lone_byte"
# subject = "codecs.utf_16_decode"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_codecs.py"
# status = "filled"
# ///
"""codecs.utf_16_decode: the low-level utf_16_decode applies its handler to a lone trailing byte: 'replace' yields the U+FFFD char with consumed 1, 'ignore' yields '' with consumed 1"""
import codecs

assert codecs.utf_16_decode(b"\x01", "replace", True) == ("�", 1)
assert codecs.utf_16_decode(b"\x01", "ignore", True) == ("", 1)

print("utf16_decode_handlers_on_lone_byte OK")
"###);
    assert_output(&out, r###"utf16_decode_handlers_on_lone_byte OK
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/codecs/utf16_le_test__test_nonbmp.py`.
#[test]
fn test_gen_behavior_std_libs_codecs_utf16_le_test__test_nonbmp() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "codecs"
# dimension = "behavior"
# case = "utf16_le_test__test_nonbmp"
# subject = "cpython.test_codecs.UTF16LETest.test_nonbmp"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_codecs.py"
# status = "filled"
# ///
# Auto-ported from CPython 3.12 test_codecs.py::UTF16LETest::test_nonbmp
"""Auto-ported test: UTF16LETest::test_nonbmp (CPython 3.12 oracle)."""


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
encoding = 'utf-16-le'
ill_formed_sequence = b'\x80\xdc'

assert '\U00010203'.encode(encoding) == b'\x00\xd8\x03\xde'

assert b'\x00\xd8\x03\xde'.decode(encoding) == '\U00010203'
print("UTF16LETest::test_nonbmp: ok")
"###);
    assert_output(&out, r###"UTF16LETest::test_nonbmp: ok
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/codecs/utf16_test__test_errors.py`.
#[test]
fn test_gen_behavior_std_libs_codecs_utf16_test__test_errors() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "codecs"
# dimension = "behavior"
# case = "utf16_test__test_errors"
# subject = "cpython.test_codecs.UTF16Test.test_errors"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_codecs.py"
# status = "filled"
# ///
# Auto-ported from CPython 3.12 test_codecs.py::UTF16Test::test_errors
"""Auto-ported test: UTF16Test::test_errors (CPython 3.12 oracle)."""


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
encoding = 'utf-16'
spamle = b'\xff\xfes\x00p\x00a\x00m\x00s\x00p\x00a\x00m\x00'
spambe = b'\xfe\xff\x00s\x00p\x00a\x00m\x00s\x00p\x00a\x00m'

try:
    codecs.utf_16_decode(b'\xff', 'strict', True)
    raise AssertionError('expected UnicodeDecodeError')
except UnicodeDecodeError:
    pass
print("UTF16Test::test_errors: ok")
"###);
    assert_output(&out, r###"UTF16Test::test_errors: ok
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/codecs/utf16_test__test_handlers.py`.
#[test]
fn test_gen_behavior_std_libs_codecs_utf16_test__test_handlers() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "codecs"
# dimension = "behavior"
# case = "utf16_test__test_handlers"
# subject = "cpython.test_codecs.UTF16Test.test_handlers"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_codecs.py"
# status = "filled"
# ///
# Auto-ported from CPython 3.12 test_codecs.py::UTF16Test::test_handlers
"""Auto-ported test: UTF16Test::test_handlers (CPython 3.12 oracle)."""


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
encoding = 'utf-16'
spamle = b'\xff\xfes\x00p\x00a\x00m\x00s\x00p\x00a\x00m\x00'
spambe = b'\xfe\xff\x00s\x00p\x00a\x00m\x00s\x00p\x00a\x00m'

assert ('�', 1) == codecs.utf_16_decode(b'\x01', 'replace', True)

assert ('', 1) == codecs.utf_16_decode(b'\x01', 'ignore', True)
print("UTF16Test::test_handlers: ok")
"###);
    assert_output(&out, r###"UTF16Test::test_handlers: ok
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/codecs/utf32_be_test__test_simple.py`.
#[test]
fn test_gen_behavior_std_libs_codecs_utf32_be_test__test_simple() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "codecs"
# dimension = "behavior"
# case = "utf32_be_test__test_simple"
# subject = "cpython.test_codecs.UTF32BETest.test_simple"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_codecs.py"
# status = "filled"
# ///
# Auto-ported from CPython 3.12 test_codecs.py::UTF32BETest::test_simple
"""Auto-ported test: UTF32BETest::test_simple (CPython 3.12 oracle)."""


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
encoding = 'utf-32-be'
ill_formed_sequence = b'\x00\x00\xdc\x80'

assert '\U00010203'.encode(encoding) == b'\x00\x01\x02\x03'
print("UTF32BETest::test_simple: ok")
"###);
    assert_output(&out, r###"UTF32BETest::test_simple: ok
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/codecs/utf32_le_test__test_simple.py`.
#[test]
fn test_gen_behavior_std_libs_codecs_utf32_le_test__test_simple() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "codecs"
# dimension = "behavior"
# case = "utf32_le_test__test_simple"
# subject = "cpython.test_codecs.UTF32LETest.test_simple"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_codecs.py"
# status = "filled"
# ///
# Auto-ported from CPython 3.12 test_codecs.py::UTF32LETest::test_simple
"""Auto-ported test: UTF32LETest::test_simple (CPython 3.12 oracle)."""


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
encoding = 'utf-32-le'
ill_formed_sequence = b'\x80\xdc\x00\x00'

assert '\U00010203'.encode(encoding) == b'\x03\x02\x01\x00'
print("UTF32LETest::test_simple: ok")
"###);
    assert_output(&out, r###"UTF32LETest::test_simple: ok
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/codecs/utf8_multibyte_roundtrip.py`.
#[test]
fn test_gen_behavior_std_libs_codecs_utf8_multibyte_roundtrip() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "codecs"
# dimension = "behavior"
# case = "utf8_multibyte_roundtrip"
# subject = "codecs.encode"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_codecs.py"
# status = "filled"
# ///
"""codecs.encode: codecs.encode/decode round-trips a multi-byte string: 'hello 日本語 café' encodes to bytes and decodes back equal under utf-8"""
import codecs

_text = "hello 日本語 café"
_encoded = codecs.encode(_text, "utf-8")
assert isinstance(_encoded, bytes), f"encode returns bytes: {type(_encoded)!r}"
_decoded = codecs.decode(_encoded, "utf-8")
assert _decoded == _text, f"utf-8 round-trip = {_decoded!r}"

print("utf8_multibyte_roundtrip OK")
"###);
    assert_output(&out, r###"utf8_multibyte_roundtrip OK
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/codecs/uu_codec_roundtrip.py`.
#[test]
fn test_gen_behavior_std_libs_codecs_uu_codec_roundtrip() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "codecs"
# dimension = "behavior"
# case = "uu_codec_roundtrip"
# subject = "codecs.encode"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_codecs.py"
# status = "filled"
# ///
"""codecs.encode: the uu_codec is a bytes->bytes transform via codecs.encode/decode: b'hello world' encodes to bytes and decodes back equal"""
import codecs

_data = b"hello world"
_uu = codecs.encode(_data, "uu_codec")
assert isinstance(_uu, bytes), "uu_codec yields bytes"
assert codecs.decode(_uu, "uu_codec") == _data, "uu round-trip"

print("uu_codec_roundtrip OK")
"###);
    assert_output(&out, r###"uu_codec_roundtrip OK
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/codecs/with_stmt_test__test_encodedfile.py`.
#[test]
fn test_gen_behavior_std_libs_codecs_with_stmt_test__test_encodedfile() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "codecs"
# dimension = "behavior"
# case = "with_stmt_test__test_encodedfile"
# subject = "cpython.test_codecs.WithStmtTest.test_encodedfile"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_codecs.py"
# status = "filled"
# ///
# Auto-ported from CPython 3.12 test_codecs.py::WithStmtTest::test_encodedfile
"""Auto-ported test: WithStmtTest::test_encodedfile (CPython 3.12 oracle)."""


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
f = io.BytesIO(b'\xc3\xbc')
with codecs.EncodedFile(f, 'latin-1', 'utf-8') as ef:

    assert ef.read() == b'\xfc'

assert f.closed
print("WithStmtTest::test_encodedfile: ok")
"###);
    assert_output(&out, r###"WithStmtTest::test_encodedfile: ok
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/codecs/zlib_codec_compresses.py`.
#[test]
fn test_gen_behavior_std_libs_codecs_zlib_codec_compresses() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "codecs"
# dimension = "behavior"
# case = "zlib_codec_compresses"
# subject = "codecs.encode"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_codecs.py"
# status = "filled"
# ///
"""codecs.encode: the zlib_codec compresses then losslessly restores: a 20x-repeated payload encodes to a strictly smaller bytes object and decodes back equal"""
import codecs

_data = b"compress me please " * 20
_z = codecs.encode(_data, "zlib_codec")
assert isinstance(_z, bytes), "zlib encoded is bytes"
assert len(_z) < len(_data), "zlib compressed smaller"
_back = codecs.decode(_z, "zlib_codec")
assert _back == _data, f"zlib round-trip = {_back!r}"

print("zlib_codec_compresses OK")
"###);
    assert_output(&out, r###"zlib_codec_compresses OK
"###);
}
