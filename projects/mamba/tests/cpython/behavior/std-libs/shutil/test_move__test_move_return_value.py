# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "shutil"
# dimension = "behavior"
# case = "test_move__test_move_return_value"
# subject = "cpython.test_shutil.TestMove.test_move_return_value"
# kind = "semantic"
# xfail = "auto-ported CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_shutil.py"
# status = "filled"
# ///
# mamba-xfail: auto-ported CPython test; mamba promotion pending
# Auto-ported from CPython 3.12 test_shutil.py::TestMove::test_move_return_value
"""Auto-ported test: TestMove::test_move_return_value (CPython 3.12 oracle)."""


import unittest
import unittest.mock
import shutil
import tempfile
import sys
import stat
import os
import os.path
import errno
import functools
import pathlib
import subprocess
import random
import string
import contextlib
import io
from shutil import make_archive, register_archive_format, unregister_archive_format, get_archive_formats, Error, unpack_archive, register_unpack_format, RegistryError, unregister_unpack_format, get_unpack_formats, SameFileError, _GiveupOnFastCopy
import tarfile
import warnings
import zipfile
from test import support
from test.support import os_helper
from test.support.os_helper import TESTFN, FakePath
from test.support import warnings_helper


try:
    import posix
except ImportError:
    posix = None

TESTFN2 = TESTFN + '2'

TESTFN_SRC = TESTFN + '_SRC'

TESTFN_DST = TESTFN + '_DST'

MACOS = sys.platform.startswith('darwin')

SOLARIS = sys.platform.startswith('sunos')

AIX = sys.platform[:3] == 'aix'

try:
    import grp
    import pwd
    UID_GID_SUPPORT = True
except ImportError:
    UID_GID_SUPPORT = False

try:
    import _winapi
except ImportError:
    _winapi = None

no_chdir = unittest.mock.patch('os.chdir', side_effect=AssertionError("shouldn't call os.chdir()"))

def _fake_rename(*args, **kwargs):
    raise OSError(getattr(errno, 'EXDEV', 18), 'Invalid cross-device link')

def mock_rename(func):

    @functools.wraps(func)
    def wrap(*args, **kwargs):
        try:
            builtin_rename = os.rename
            os.rename = _fake_rename
            return func(*args, **kwargs)
        finally:
            os.rename = builtin_rename
    return wrap

def create_file(path, content=b''):
    """Write *content* to a file located at *path*.

    If *path* is a tuple instead of a string, os.path.join will be used to
    make a path.
    """
    if isinstance(path, tuple):
        path = os.path.join(*path)
    if isinstance(content, str):
        content = content.encode()
    with open(path, 'xb') as fp:
        fp.write(content)

def write_test_file(path, size):
    """Create a test file with an arbitrary size and random text content."""

    def chunks(total, step):
        assert total >= step
        while total > step:
            yield step
            total -= step
        if total:
            yield total
    bufsize = min(size, 8192)
    chunk = b''.join([random.choice(string.ascii_letters).encode() for i in range(bufsize)])
    with open(path, 'wb') as f:
        for csize in chunks(size, bufsize):
            f.write(chunk)
    assert os.path.getsize(path) == size

def read_file(path, binary=False):
    """Return contents from a file located at *path*.

    If *path* is a tuple instead of a string, os.path.join will be used to
    make a path.  If *binary* is true, the file will be opened in binary
    mode.
    """
    if isinstance(path, tuple):
        path = os.path.join(*path)
    mode = 'rb' if binary else 'r'
    encoding = None if binary else 'utf-8'
    with open(path, mode, encoding=encoding) as fp:
        return fp.read()

def rlistdir(path):
    res = []
    for name in sorted(os.listdir(path)):
        p = os.path.join(path, name)
        if os.path.isdir(p) and (not os.path.islink(p)):
            res.append(name + '/')
            for n in rlistdir(p):
                res.append(name + '/' + n)
        else:
            res.append(name)
    return res

def supports_file2file_sendfile():
    if not hasattr(os, 'sendfile'):
        return False
    srcname = None
    dstname = None
    try:
        with tempfile.NamedTemporaryFile('wb', dir=os.getcwd(), delete=False) as f:
            srcname = f.name
            f.write(b'0123456789')
        with open(srcname, 'rb') as src:
            with tempfile.NamedTemporaryFile('wb', dir=os.getcwd(), delete=False) as dst:
                dstname = dst.name
                infd = src.fileno()
                outfd = dst.fileno()
                try:
                    os.sendfile(outfd, infd, 0, 2)
                except OSError:
                    return False
                else:
                    return True
    finally:
        if srcname is not None:
            os_helper.unlink(srcname)
        if dstname is not None:
            os_helper.unlink(dstname)

SUPPORTS_SENDFILE = supports_file2file_sendfile()

def _maxdataOK():
    if AIX and sys.maxsize == 2147483647:
        hdrs = subprocess.getoutput('/usr/bin/dump -o %s' % sys.executable)
        maxdata = hdrs.split('\n')[-1].split()[1]
        return int(maxdata, 16) >= 536870912
    else:
        return True

class BaseTest:

    def mkdtemp(self, prefix=None):
        """Create a temporary directory that will be cleaned up.

        Returns the path of the directory.
        """
        d = tempfile.mkdtemp(prefix=prefix, dir=os.getcwd())
        self.addCleanup(os_helper.rmtree, d)
        return d


# --- test body ---
def _check_move_dir(src, dst, real_dst):
    contents = sorted(os.listdir(src))
    shutil.move(src, dst)

    assert contents == sorted(os.listdir(real_dst))

    assert not os.path.exists(src)

def _check_move_file(src, dst, real_dst):
    with open(src, 'rb') as f:
        contents = f.read()
    shutil.move(src, dst)
    with open(real_dst, 'rb') as f:

        assert contents == f.read()

    assert not os.path.exists(src)

def _test_move_symlink_to_dir_into_dir(dst):
    src = os.path.join(self_src_dir, 'linktodir')
    dst_link = os.path.join(self_dst_dir, 'linktodir')
    os.symlink(self_dst_dir, src, target_is_directory=True)
    shutil.move(src, dst)

    assert os.path.islink(dst_link)

    assert os.path.samefile(self_dst_dir, dst_link)

    assert not os.path.exists(src)
    os.symlink(self_dst_dir, src, target_is_directory=True)
    try:
        shutil.move(src, dst)
        raise AssertionError('expected shutil.Error')
    except shutil.Error:
        pass

    assert os.path.samefile(self_dst_dir, dst_link)

    assert os.path.exists(src)

def mkdtemp(prefix=None):
    """Create a temporary directory that will be cleaned up.

        Returns the path of the directory.
        """
    d = tempfile.mkdtemp(prefix=prefix, dir=os.getcwd())
    pass
    return d
filename = 'foo'
self_src_dir = mkdtemp()
self_dst_dir = mkdtemp()
self_src_file = os.path.join(self_src_dir, filename)
self_dst_file = os.path.join(self_dst_dir, filename)
create_file(self_src_file, b'spam')
rv = shutil.move(self_src_file, self_dst_dir)

assert rv == os.path.join(self_dst_dir, os.path.basename(self_src_file))
print("TestMove::test_move_return_value: ok")
