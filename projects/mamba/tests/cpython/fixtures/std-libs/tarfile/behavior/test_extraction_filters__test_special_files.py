# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "tarfile"
# dimension = "behavior"
# case = "test_extraction_filters__test_special_files"
# subject = "cpython.test_tarfile.TestExtractionFilters.test_special_files"
# kind = "semantic"
# xfail = "auto-ported CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_tarfile.py"
# status = "filled"
# ///
# mamba-xfail: auto-ported CPython test; mamba promotion pending
# Auto-ported from CPython 3.12 test_tarfile.py::TestExtractionFilters::test_special_files
"""Auto-ported test: TestExtractionFilters::test_special_files (CPython 3.12 oracle)."""


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
outerdir = pathlib.Path(TEMPDIR) / 'outerdir'
destdir = outerdir / 'dest'

def check_context(tar, filter, *, check_flag=True):
    """Extracts `tar` to `self.destdir` and allows checking the result

        If an error occurs, it must be checked using `expect_exception`

        Otherwise, all resulting files must be checked using `expect_file`,
        except the destination directory itself and parent directories of
        other files.
        When checking directories, do so before their contents.

        A file called 'flag' is made in outerdir (i.e. outside destdir)
        before extraction; it should not be altered nor should its contents
        be read/copied.
        """
    with os_helper.temp_dir(outerdir):
        flag_path = outerdir / 'flag'
        flag_path.write_text('capture me')
        try:
            tar.extractall(destdir, filter=filter)
        except Exception as exc:
            self_raised_exception = exc
            self_reraise_exception = True
            self_expected_paths = set()
        else:
            self_raised_exception = None
            self_reraise_exception = False
            self_expected_paths = set(outerdir.glob('**/*'))
            self_expected_paths.discard(destdir)
            self_expected_paths.discard(flag_path)
        try:
            yield self
        finally:
            tar.close()
        if self_reraise_exception:
            raise self_raised_exception

        assert self_expected_paths == set()
        if check_flag:

            assert flag_path.read_text() == 'capture me'
        else:
            assert filter == 'fully_trusted'

def expect_any_tree(name):
    """Check a directory; forget about its contents."""
    tree_path = (destdir / name).resolve()
    expect_file(tree_path, type=tarfile.DIRTYPE)
    self_expected_paths = {p for p in self_expected_paths if tree_path not in p.parents}

def expect_exception(exc_type, message_re='.'):
    try:
        if self_raised_exception is not None:
            raise self_raised_exception
        raise AssertionError('expected exc_type')
    except exc_type as _aR_e:
        import re as _re_aR
        assert _re_aR.search(message_re, str(_aR_e))
    self_reraise_exception = False
    return self_raised_exception

def expect_file(name, type=None, symlink_to=None, mode=None, size=None, content=None):
    """Check a single file. See check_context."""
    if self_raised_exception:
        raise self_raised_exception
    path = pathlib.Path(os.path.normpath(destdir / name))

    assert path in self_expected_paths
    self_expected_paths.remove(path)
    if mode is not None and os_helper.can_chmod() and (os.name != 'nt'):
        got = stat.filemode(stat.S_IMODE(path.stat().st_mode))

        assert got == mode
    if type is None and isinstance(name, str) and name.endswith('/'):
        type = tarfile.DIRTYPE
    if symlink_to is not None:
        got = (destdir / name).readlink()
        expected = pathlib.Path(symlink_to)
        try:
            if expected != got:

                assert got.samefile(expected)
        except Exception as e:
            e.add_note(f'expected={expected!r}, got={got!r}')
            raise
    elif type == tarfile.REGTYPE or type is None:

        assert path.is_file()
    elif type == tarfile.DIRTYPE:

        assert path.is_dir()
    elif type == tarfile.FIFOTYPE:

        assert path.is_fifo()
    elif type == tarfile.SYMTYPE:

        assert path.is_symlink()
    else:
        raise NotImplementedError(type)
    if size is not None:

        assert path.stat().st_size == size
    if content is not None:

        assert path.read_text() == content
    for parent in path.parents:
        self_expected_paths.discard(parent)
for special_type in (tarfile.FIFOTYPE, tarfile.CHRTYPE, tarfile.BLKTYPE):
    tarinfo = tarfile.TarInfo('foo')
    tarinfo.type = special_type
    trusted = tarfile.fully_trusted_filter(tarinfo, '')

    assert trusted is tarinfo
    tar = tarfile.tar_filter(tarinfo, '')

    assert tar.type == special_type
    try:
        tarfile.data_filter(tarinfo, '')
        raise AssertionError('expected tarfile.SpecialFileError')
    except tarfile.SpecialFileError as _aR_e:
        import types as _types_aR
        cm = _types_aR.SimpleNamespace(exception=_aR_e)

    assert isinstance(cm.exception.tarinfo, tarfile.TarInfo)

    assert cm.exception.tarinfo.name == 'foo'
print("TestExtractionFilters::test_special_files: ok")
