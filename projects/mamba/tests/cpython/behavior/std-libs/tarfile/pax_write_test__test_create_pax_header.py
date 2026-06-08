# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "tarfile"
# dimension = "behavior"
# case = "pax_write_test__test_create_pax_header"
# subject = "cpython.test_tarfile.PaxWriteTest.test_create_pax_header"
# kind = "semantic"
# xfail = "auto-ported CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_tarfile.py"
# status = "filled"
# ///
# mamba-xfail: auto-ported CPython test; mamba promotion pending
# Auto-ported from CPython 3.12 test_tarfile.py::PaxWriteTest::test_create_pax_header
"""Auto-ported test: PaxWriteTest::test_create_pax_header (CPython 3.12 oracle)."""


import errno
import sys
import os
import io
from hashlib import sha256
from contextlib import contextmanager, ExitStack
from random import Random
import pathlib
import shutil
import re
import warnings
import stat
import unittest
import unittest.mock
import tarfile
from test import archiver_tests
from test import support
from test.support import os_helper
from test.support import script_helper
from test.support import warnings_helper


try:
    import gzip
except ImportError:
    gzip = None

try:
    import zlib
except ImportError:
    zlib = None

try:
    import bz2
except ImportError:
    bz2 = None

try:
    import lzma
except ImportError:
    lzma = None

def sha256sum(data):
    return sha256(data).hexdigest()

TEMPDIR = os.path.abspath(os_helper.TESTFN) + '-tardir'

tarextdir = TEMPDIR + '-extract-test'

tarname = support.findfile('testtar.tar')

gzipname = os.path.join(TEMPDIR, 'testtar.tar.gz')

bz2name = os.path.join(TEMPDIR, 'testtar.tar.bz2')

xzname = os.path.join(TEMPDIR, 'testtar.tar.xz')

tmpname = os.path.join(TEMPDIR, 'tmp.tar')

dotlessname = os.path.join(TEMPDIR, 'testtar')

sha256_regtype = 'e09e4bc8b3c9d9177e77256353b36c159f5f040531bbd4b024a8f9b9196c71ce'

sha256_sparse = '4f05a776071146756345ceee937b33fc5644f5a96b9780d1c7d6a32cdf164d7b'

class TarTest:
    tarname = tarname
    suffix = ''
    open = io.FileIO
    taropen = tarfile.TarFile.taropen

    @property
    def mode(self):
        return self.prefix + self.suffix

@support.requires_gzip()
class GzipTest:
    tarname = gzipname
    suffix = 'gz'
    open = gzip.GzipFile if gzip else None
    taropen = tarfile.TarFile.gzopen

@support.requires_bz2()
class Bz2Test:
    tarname = bz2name
    suffix = 'bz2'
    open = bz2.BZ2File if bz2 else None
    taropen = tarfile.TarFile.bz2open

@support.requires_lzma()
class LzmaTest:
    tarname = xzname
    suffix = 'xz'
    open = lzma.LZMAFile if lzma else None
    taropen = tarfile.TarFile.xzopen

class ReadTest(TarTest):
    prefix = 'r:'

    def setUp(self):
        self.tar = tarfile.open(self.tarname, mode=self.mode, encoding='iso8859-1')

    def tearDown(self):
        self.tar.close()

class _CompressedWriteTest(TarTest):
    source = b'And we move to Bristol where they have a special, ' + b'Very Silly candidate'

    def _compressed_tar(self, compresslevel):
        fobj = io.BytesIO()
        with tarfile.open(tmpname, self.mode, fobj, compresslevel=compresslevel) as tarfl:
            tarfl.addfile(tarfile.TarInfo('foo'), io.BytesIO(self.source))
        return fobj

    def _test_bz2_header(self, compresslevel):
        fobj = self._compressed_tar(compresslevel)
        self.assertEqual(fobj.getvalue()[0:10], b'BZh%d1AY&SY' % compresslevel)

    def _test_gz_header(self, compresslevel):
        fobj = self._compressed_tar(compresslevel)
        self.assertEqual(fobj.getvalue()[:3], b'\x1f\x8b\x08')

def root_is_uid_gid_0():
    try:
        import pwd, grp
    except ImportError:
        return False
    if pwd.getpwuid(0)[0] != 'root':
        return False
    if grp.getgrgid(0)[0] != 'root':
        return False
    return True

def _filemode_to_int(mode):
    """Inverse of `stat.filemode` (for permission bits)

    Using mode strings rather than numbers makes the later tests more readable.
    """
    str_mode = mode[1:]
    result = {'r': stat.S_IRUSR, '-': 0}[str_mode[0]] | {'w': stat.S_IWUSR, '-': 0}[str_mode[1]] | {'x': stat.S_IXUSR, '-': 0, 's': stat.S_IXUSR | stat.S_ISUID, 'S': stat.S_ISUID}[str_mode[2]] | {'r': stat.S_IRGRP, '-': 0}[str_mode[3]] | {'w': stat.S_IWGRP, '-': 0}[str_mode[4]] | {'x': stat.S_IXGRP, '-': 0, 's': stat.S_IXGRP | stat.S_ISGID, 'S': stat.S_ISGID}[str_mode[5]] | {'r': stat.S_IROTH, '-': 0}[str_mode[6]] | {'w': stat.S_IWOTH, '-': 0}[str_mode[7]] | {'x': stat.S_IXOTH, '-': 0, 't': stat.S_IXOTH | stat.S_ISVTX, 'T': stat.S_ISVTX}[str_mode[8]]
    assert stat.filemode(result)[1:] == mode[1:]
    return result

class ArchiveMaker:
    """Helper to create a tar file with specific contents

    Usage:

        with ArchiveMaker() as t:
            t.add('filename', ...)

        with t.open() as tar:
            ... # `tar` is now a TarFile with 'filename' in it!
    """

    def __init__(self):
        self.bio = io.BytesIO()

    def __enter__(self):
        self.tar_w = tarfile.TarFile(mode='w', fileobj=self.bio)
        return self

    def __exit__(self, *exc):
        self.tar_w.close()
        self.contents = self.bio.getvalue()
        self.bio = None

    def add(self, name, *, type=None, symlink_to=None, hardlink_to=None, mode=None, size=None, content=None, **kwargs):
        """Add a member to the test archive. Call within `with`.

        Provides many shortcuts:
        - default `type` is based on symlink_to, hardlink_to, and trailing `/`
          in name (which is stripped)
        - size & content defaults are based on each other
        - content can be str or bytes
        - mode should be textual ('-rwxrwxrwx')

        (add more! this is unstable internal test-only API)
        """
        name = str(name)
        tarinfo = tarfile.TarInfo(name).replace(**kwargs)
        if content is not None:
            if isinstance(content, str):
                content = content.encode()
            size = len(content)
        if size is not None:
            tarinfo.size = size
            if content is None:
                content = bytes(tarinfo.size)
        if mode:
            tarinfo.mode = _filemode_to_int(mode)
        if symlink_to is not None:
            type = tarfile.SYMTYPE
            tarinfo.linkname = str(symlink_to)
        if hardlink_to is not None:
            type = tarfile.LNKTYPE
            tarinfo.linkname = str(hardlink_to)
        if name.endswith('/') and type is None:
            type = tarfile.DIRTYPE
        if type is not None:
            tarinfo.type = type
        if tarinfo.isreg():
            fileobj = io.BytesIO(content)
        else:
            fileobj = None
        self.tar_w.addfile(tarinfo, fileobj)

    def open(self, **kwargs):
        """Open the resulting archive as TarFile. Call after `with`."""
        bio = io.BytesIO(self.contents)
        return tarfile.open(fileobj=bio, **kwargs)

if support.is_wasi:

    def symlink_test(f):
        return unittest.skip('WASI: Skip symlink test for now')(f)
else:

    def symlink_test(f):
        return f

def setUpModule():
    os_helper.unlink(TEMPDIR)
    os.makedirs(TEMPDIR)
    global testtarnames
    testtarnames = [tarname]
    with open(tarname, 'rb') as fobj:
        data = fobj.read()
    for c in (GzipTest, Bz2Test, LzmaTest):
        if c.open:
            os_helper.unlink(c.tarname)
            testtarnames.append(c.tarname)
            with c.open(c.tarname, 'wb') as tar:
                tar.write(data)

def tearDownModule():
    if os.path.exists(TEMPDIR):
        os_helper.rmtree(TEMPDIR)


# --- test body ---
def _calc_size(name, link=None):
    count = 512
    if len(name) > tarfile.LENGTH_NAME:
        count += 512
        count += _length(name)
    if link is not None and len(link) > tarfile.LENGTH_LINK:
        count += 512
        count += _length(link)
    return count

def _length(s):
    blocks = len(s) // 512 + 1
    return blocks * 512

def _test(name, link=None):
    tarinfo = tarfile.TarInfo(name)
    if link:
        tarinfo.linkname = link
        tarinfo.type = tarfile.LNKTYPE
    tar = tarfile.open(tmpname, 'w', format=tarfile.PAX_FORMAT)
    try:
        tar.addfile(tarinfo)
    finally:
        tar.close()
    tar = tarfile.open(tmpname)
    try:
        if link:
            l = tar.getmembers()[0].linkname

            assert link == l
        else:
            n = tar.getmembers()[0].name

            assert name == n
    finally:
        tar.close()
t = tarfile.TarInfo()
t.name = 'foo'
t.mtime = 1000.1
t.size = 100
t.uid = 123
t.gid = 124
info = t.get_info()
header = t.create_pax_header(info, encoding='iso8859-1')

assert info['name'] == 'foo'

assert isinstance(info['mtime'], int)

assert info['mtime'] == 1000

assert info['size'] == 100

assert info['uid'] == 123

assert info['gid'] == 124

assert header == b'././@PaxHeader' + bytes(86) + b'0000000\x000000000\x000000000\x0000000000020\x0000000000000\x00010205\x00 x' + bytes(100) + b'ustar\x0000' + bytes(247) + b'16 mtime=1000.1\n' + bytes(496) + b'foo' + bytes(97) + b'0000644\x000000173\x000000174\x0000000000144\x0000000001750\x00006516\x00 0' + bytes(100) + b'ustar\x0000' + bytes(247)
t = tarfile.TarInfo()
t.name = 'foo㍴'
t.mtime = 10 ** 10
t.size = 10 ** 10
t.uid = 8 ** 8
t.gid = 8 ** 8 + 1
info = t.get_info()
header = t.create_pax_header(info, encoding='iso8859-1')

assert info['name'] == 'foo㍴'

assert info['mtime'] == 0

assert info['size'] == 0

assert info['uid'] == 0

assert info['gid'] == 0

assert header == b'././@PaxHeader' + bytes(86) + b'0000000\x000000000\x000000000\x0000000000130\x0000000000000\x00010207\x00 x' + bytes(100) + b'ustar\x0000' + bytes(247) + b'15 path=foo\xe3\x8d\xb4\n16 uid=16777216\n' + b'16 gid=16777217\n20 size=10000000000\n' + b'21 mtime=10000000000\n' + bytes(424) + b'foo?' + bytes(96) + b'0000644\x000000000\x000000000\x0000000000000\x0000000000000\x00006540\x00 0' + bytes(100) + b'ustar\x0000' + bytes(247)
print("PaxWriteTest::test_create_pax_header: ok")
