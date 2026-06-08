# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "bz2"
# dimension = "behavior"
# case = "bz2_decompressor_test__test_decompressor_inputbuf_3"
# subject = "cpython.test_bz2.BZ2DecompressorTest.test_decompressor_inputbuf_3"
# kind = "semantic"
# xfail = "auto-ported CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_bz2.py"
# status = "filled"
# ///
# mamba-xfail: auto-ported CPython test; mamba promotion pending
# Auto-ported from CPython 3.12 test_bz2.py::BZ2DecompressorTest::test_decompressor_inputbuf_3
"""Auto-ported test: BZ2DecompressorTest::test_decompressor_inputbuf_3 (CPython 3.12 oracle)."""


from test import support
from test.support import bigmemtest, _4G
import array
import unittest
import io
from io import BytesIO, DEFAULT_BUFFER_SIZE
import os
import pickle
import glob
import tempfile
import random
import shutil
import subprocess
import threading
from test.support import import_helper
from test.support import threading_helper
from test.support.os_helper import unlink, FakePath
import _compression
import sys
from bz2 import BZ2File, BZ2Compressor, BZ2Decompressor


bz2 = import_helper.import_module('bz2')

has_cmdline_bunzip2 = None

def ext_decompress(data):
    global has_cmdline_bunzip2
    if has_cmdline_bunzip2 is None:
        has_cmdline_bunzip2 = bool(shutil.which('bunzip2'))
    if has_cmdline_bunzip2:
        return subprocess.check_output(['bunzip2'], input=data)
    else:
        return bz2.decompress(data)

class BaseTest(unittest.TestCase):
    """Base for other testcases."""
    TEXT_LINES = [b'root:x:0:0:root:/root:/bin/bash\n', b'bin:x:1:1:bin:/bin:\n', b'daemon:x:2:2:daemon:/sbin:\n', b'adm:x:3:4:adm:/var/adm:\n', b'lp:x:4:7:lp:/var/spool/lpd:\n', b'sync:x:5:0:sync:/sbin:/bin/sync\n', b'shutdown:x:6:0:shutdown:/sbin:/sbin/shutdown\n', b'halt:x:7:0:halt:/sbin:/sbin/halt\n', b'mail:x:8:12:mail:/var/spool/mail:\n', b'news:x:9:13:news:/var/spool/news:\n', b'uucp:x:10:14:uucp:/var/spool/uucp:\n', b'operator:x:11:0:operator:/root:\n', b'games:x:12:100:games:/usr/games:\n', b'gopher:x:13:30:gopher:/usr/lib/gopher-data:\n', b'ftp:x:14:50:FTP User:/var/ftp:/bin/bash\n', b'nobody:x:65534:65534:Nobody:/home:\n', b'postfix:x:100:101:postfix:/var/spool/postfix:\n', b'niemeyer:x:500:500::/home/niemeyer:/bin/bash\n', b'postgres:x:101:102:PostgreSQL Server:/var/lib/pgsql:/bin/bash\n', b'mysql:x:102:103:MySQL server:/var/lib/mysql:/bin/bash\n', b'www:x:103:104::/var/www:/bin/false\n']
    TEXT = b''.join(TEXT_LINES)
    DATA = b'BZh91AY&SY.\xc8N\x18\x00\x01>_\x80\x00\x10@\x02\xff\xf0\x01\x07n\x00?\xe7\xff\xe00\x01\x99\xaa\x00\xc0\x03F\x86\x8c#&\x83F\x9a\x03\x06\xa6\xd0\xa6\x93M\x0fQ\xa7\xa8\x06\x804hh\x12$\x11\xa4i4\xf14S\xd2<Q\xb5\x0fH\xd3\xd4\xdd\xd5\x87\xbb\xf8\x94\r\x8f\xafI\x12\xe1\xc9\xf8/E\x00pu\x89\x12]\xc9\xbbDL\nQ\x0e\t1\x12\xdf\xa0\xc0\x97\xac2O9\x89\x13\x94\x0e\x1c7\x0ed\x95I\x0c\xaaJ\xa4\x18L\x10\x05#\x9c\xaf\xba\xbc/\x97\x8a#C\xc8\xe1\x8cW\xf9\xe2\xd0\xd6M\xa7\x8bXa<e\x84t\xcbL\xb3\xa7\xd9\xcd\xd1\xcb\x84.\xaf\xb3\xab\xab\xad`n}\xa0lh\tE,\x8eZ\x15\x17VH>\x88\xe5\xcd9gd6\x0b\n\xe9\x9b\xd5\x8a\x99\xf7\x08.K\x8ev\xfb\xf7xw\xbb\xdf\xa1\x92\xf1\xdd|/";\xa2\xba\x9f\xd5\xb1#A\xb6\xf6\xb3o\xc9\xc5y\\\xebO\xe7\x85\x9a\xbc\xb6f8\x952\xd5\xd7"%\x89>V,\xf7\xa6z\xe2\x9f\xa3\xdf\x11\x11"\xd6E)I\xa9\x13^\xca\xf3r\xd0\x03U\x922\xf26\xec\xb6\xed\x8b\xc3U\x13\x9d\xc5\x170\xa4\xfa^\x92\xacDF\x8a\x97\xd6\x19\xfe\xdd\xb8\xbd\x1a\x9a\x19\xa3\x80ankR\x8b\xe5\xd83]\xa9\xc6\x08\x82f\xf6\xb9"6l$\xb8j@\xc0\x8a\xb0l1..\xbak\x83ls\x15\xbc\xf4\xc1\x13\xbe\xf8E\xb8\x9d\r\xa8\x9dk\x84\xd3n\xfa\xacQ\x07\xb1%y\xaav\xb4\x08\xe0z\x1b\x16\xf5\x04\xe9\xcc\xb9\x08z\x1en7.G\xfc]\xc9\x14\xe1B@\xbb!8`'
    EMPTY_DATA = b'BZh9\x17rE8P\x90\x00\x00\x00\x00'
    BAD_DATA = b'this is not a valid bzip2 file'
    test_size = 0
    BIG_TEXT = bytearray(128 * 1024)
    for fname in glob.glob(os.path.join(glob.escape(os.path.dirname(__file__)), '*.py')):
        with open(fname, 'rb') as fh:
            test_size += fh.readinto(memoryview(BIG_TEXT)[test_size:])
        if test_size > 128 * 1024:
            break
    BIG_DATA = bz2.compress(BIG_TEXT, compresslevel=1)

    def setUp(self):
        fd, self.filename = tempfile.mkstemp()
        os.close(fd)

    def tearDown(self):
        unlink(self.filename)

class BZ2CompressorTest(BaseTest):

    def testCompress(self):
        bz2c = BZ2Compressor()
        self.assertRaises(TypeError, bz2c.compress)
        data = bz2c.compress(self.TEXT)
        data += bz2c.flush()
        self.assertEqual(ext_decompress(data), self.TEXT)

    def testCompressEmptyString(self):
        bz2c = BZ2Compressor()
        data = bz2c.compress(b'')
        data += bz2c.flush()
        self.assertEqual(data, self.EMPTY_DATA)

    def testCompressChunks10(self):
        bz2c = BZ2Compressor()
        n = 0
        data = b''
        while True:
            str = self.TEXT[n * 10:(n + 1) * 10]
            if not str:
                break
            data += bz2c.compress(str)
            n += 1
        data += bz2c.flush()
        self.assertEqual(ext_decompress(data), self.TEXT)

    @support.skip_if_pgo_task
    @bigmemtest(size=_4G + 100, memuse=2)
    def testCompress4G(self, size):
        bz2c = BZ2Compressor()
        data = b'x' * size
        try:
            compressed = bz2c.compress(data)
            compressed += bz2c.flush()
        finally:
            data = None
        data = bz2.decompress(compressed)
        try:
            self.assertEqual(len(data), size)
            self.assertEqual(len(data.strip(b'x')), 0)
        finally:
            data = None

    def testPickle(self):
        for proto in range(pickle.HIGHEST_PROTOCOL + 1):
            with self.assertRaises(TypeError):
                pickle.dumps(BZ2Compressor(), proto)

class CompressDecompressTest(BaseTest):

    def testCompress(self):
        data = bz2.compress(self.TEXT)
        self.assertEqual(ext_decompress(data), self.TEXT)

    def testCompressEmptyString(self):
        text = bz2.compress(b'')
        self.assertEqual(text, self.EMPTY_DATA)

    def testDecompress(self):
        text = bz2.decompress(self.DATA)
        self.assertEqual(text, self.TEXT)

    def testDecompressEmpty(self):
        text = bz2.decompress(b'')
        self.assertEqual(text, b'')

    def testDecompressToEmptyString(self):
        text = bz2.decompress(self.EMPTY_DATA)
        self.assertEqual(text, b'')

    def testDecompressIncomplete(self):
        self.assertRaises(ValueError, bz2.decompress, self.DATA[:-10])

    def testDecompressBadData(self):
        self.assertRaises(OSError, bz2.decompress, self.BAD_DATA)

    def testDecompressMultiStream(self):
        text = bz2.decompress(self.DATA * 5)
        self.assertEqual(text, self.TEXT * 5)

    def testDecompressTrailingJunk(self):
        text = bz2.decompress(self.DATA + self.BAD_DATA)
        self.assertEqual(text, self.TEXT)

    def testDecompressMultiStreamTrailingJunk(self):
        text = bz2.decompress(self.DATA * 5 + self.BAD_DATA)
        self.assertEqual(text, self.TEXT * 5)

def tearDownModule():
    support.reap_children()


# --- test body ---
TEXT_LINES = [b'root:x:0:0:root:/root:/bin/bash\n', b'bin:x:1:1:bin:/bin:\n', b'daemon:x:2:2:daemon:/sbin:\n', b'adm:x:3:4:adm:/var/adm:\n', b'lp:x:4:7:lp:/var/spool/lpd:\n', b'sync:x:5:0:sync:/sbin:/bin/sync\n', b'shutdown:x:6:0:shutdown:/sbin:/sbin/shutdown\n', b'halt:x:7:0:halt:/sbin:/sbin/halt\n', b'mail:x:8:12:mail:/var/spool/mail:\n', b'news:x:9:13:news:/var/spool/news:\n', b'uucp:x:10:14:uucp:/var/spool/uucp:\n', b'operator:x:11:0:operator:/root:\n', b'games:x:12:100:games:/usr/games:\n', b'gopher:x:13:30:gopher:/usr/lib/gopher-data:\n', b'ftp:x:14:50:FTP User:/var/ftp:/bin/bash\n', b'nobody:x:65534:65534:Nobody:/home:\n', b'postfix:x:100:101:postfix:/var/spool/postfix:\n', b'niemeyer:x:500:500::/home/niemeyer:/bin/bash\n', b'postgres:x:101:102:PostgreSQL Server:/var/lib/pgsql:/bin/bash\n', b'mysql:x:102:103:MySQL server:/var/lib/mysql:/bin/bash\n', b'www:x:103:104::/var/www:/bin/false\n']
TEXT = b''.join(TEXT_LINES)
DATA = b'BZh91AY&SY.\xc8N\x18\x00\x01>_\x80\x00\x10@\x02\xff\xf0\x01\x07n\x00?\xe7\xff\xe00\x01\x99\xaa\x00\xc0\x03F\x86\x8c#&\x83F\x9a\x03\x06\xa6\xd0\xa6\x93M\x0fQ\xa7\xa8\x06\x804hh\x12$\x11\xa4i4\xf14S\xd2<Q\xb5\x0fH\xd3\xd4\xdd\xd5\x87\xbb\xf8\x94\r\x8f\xafI\x12\xe1\xc9\xf8/E\x00pu\x89\x12]\xc9\xbbDL\nQ\x0e\t1\x12\xdf\xa0\xc0\x97\xac2O9\x89\x13\x94\x0e\x1c7\x0ed\x95I\x0c\xaaJ\xa4\x18L\x10\x05#\x9c\xaf\xba\xbc/\x97\x8a#C\xc8\xe1\x8cW\xf9\xe2\xd0\xd6M\xa7\x8bXa<e\x84t\xcbL\xb3\xa7\xd9\xcd\xd1\xcb\x84.\xaf\xb3\xab\xab\xad`n}\xa0lh\tE,\x8eZ\x15\x17VH>\x88\xe5\xcd9gd6\x0b\n\xe9\x9b\xd5\x8a\x99\xf7\x08.K\x8ev\xfb\xf7xw\xbb\xdf\xa1\x92\xf1\xdd|/";\xa2\xba\x9f\xd5\xb1#A\xb6\xf6\xb3o\xc9\xc5y\\\xebO\xe7\x85\x9a\xbc\xb6f8\x952\xd5\xd7"%\x89>V,\xf7\xa6z\xe2\x9f\xa3\xdf\x11\x11"\xd6E)I\xa9\x13^\xca\xf3r\xd0\x03U\x922\xf26\xec\xb6\xed\x8b\xc3U\x13\x9d\xc5\x170\xa4\xfa^\x92\xacDF\x8a\x97\xd6\x19\xfe\xdd\xb8\xbd\x1a\x9a\x19\xa3\x80ankR\x8b\xe5\xd83]\xa9\xc6\x08\x82f\xf6\xb9"6l$\xb8j@\xc0\x8a\xb0l1..\xbak\x83ls\x15\xbc\xf4\xc1\x13\xbe\xf8E\xb8\x9d\r\xa8\x9dk\x84\xd3n\xfa\xacQ\x07\xb1%y\xaav\xb4\x08\xe0z\x1b\x16\xf5\x04\xe9\xcc\xb9\x08z\x1en7.G\xfc]\xc9\x14\xe1B@\xbb!8`'
EMPTY_DATA = b'BZh9\x17rE8P\x90\x00\x00\x00\x00'
BAD_DATA = b'this is not a valid bzip2 file'
test_size = 0
BIG_TEXT = bytearray(128 * 1024)
BIG_DATA = bz2.compress(BIG_TEXT, compresslevel=1)
fd, self_filename = tempfile.mkstemp()
os.close(fd)
bzd = BZ2Decompressor()
out = []
out.append(bzd.decompress(DATA[:200], 5))
out.append(bzd.decompress(DATA[200:300], 5))
out.append(bzd.decompress(DATA[300:]))

assert b''.join(out) == TEXT
print("BZ2DecompressorTest::test_decompressor_inputbuf_3: ok")
