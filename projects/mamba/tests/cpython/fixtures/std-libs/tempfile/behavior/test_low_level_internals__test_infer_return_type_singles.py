# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "tempfile"
# dimension = "behavior"
# case = "test_low_level_internals__test_infer_return_type_singles"
# subject = "cpython.test_tempfile.TestLowLevelInternals.test_infer_return_type_singles"
# kind = "semantic"
# xfail = "auto-ported CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_tempfile.py"
# status = "filled"
# ///
# mamba-xfail: auto-ported CPython test; mamba promotion pending
# Auto-ported from CPython 3.12 test_tempfile.py::TestLowLevelInternals::test_infer_return_type_singles
"""Auto-ported test: TestLowLevelInternals::test_infer_return_type_singles (CPython 3.12 oracle)."""


import tempfile
import errno
import io
import os
import pathlib
import sys
import re
import warnings
import contextlib
import stat
import types
import weakref
import gc
import shutil
import subprocess
from unittest import mock
import unittest
from test import support
from test.support import os_helper
from test.support import script_helper
from test.support import warnings_helper


has_textmode = tempfile._text_openflags != tempfile._bin_openflags

has_spawnl = hasattr(os, 'spawnl')

if sys.platform.startswith('openbsd'):
    TEST_FILES = 48
else:
    TEST_FILES = 100

class BaseTestCase(unittest.TestCase):
    str_check = re.compile('^[a-z0-9_-]{8}$')
    b_check = re.compile(b'^[a-z0-9_-]{8}$')

    def setUp(self):
        self.enterContext(warnings_helper.check_warnings())
        warnings.filterwarnings('ignore', category=RuntimeWarning, message='mktemp', module=__name__)

    def nameCheck(self, name, dir, pre, suf):
        ndir, nbase = os.path.split(name)
        npre = nbase[:len(pre)]
        nsuf = nbase[len(nbase) - len(suf):]
        if dir is not None:
            self.assertIs(type(name), str if type(dir) is str or isinstance(dir, os.PathLike) else bytes, 'unexpected return type')
        if pre is not None:
            self.assertIs(type(name), str if type(pre) is str else bytes, 'unexpected return type')
        if suf is not None:
            self.assertIs(type(name), str if type(suf) is str else bytes, 'unexpected return type')
        if (dir, pre, suf) == (None, None, None):
            self.assertIs(type(name), str, 'default return type must be str')
        self.assertEqual(os.path.abspath(ndir), os.path.abspath(dir), 'file %r not in directory %r' % (name, dir))
        self.assertEqual(npre, pre, 'file %r does not begin with %r' % (nbase, pre))
        self.assertEqual(nsuf, suf, 'file %r does not end with %r' % (nbase, suf))
        nbase = nbase[len(pre):len(nbase) - len(suf)]
        check = self.str_check if isinstance(nbase, str) else self.b_check
        self.assertTrue(check.match(nbase), 'random characters %r do not match %r' % (nbase, check.pattern))

@contextlib.contextmanager
def _inside_empty_temp_dir():
    dir = tempfile.mkdtemp()
    try:
        with support.swap_attr(tempfile, 'tempdir', dir):
            yield
    finally:
        os_helper.rmtree(dir)

def _mock_candidate_names(*names):
    return support.swap_attr(tempfile, '_get_candidate_names', lambda: iter(names))

if tempfile.NamedTemporaryFile is not tempfile.TemporaryFile:

    class TestTemporaryFile(BaseTestCase):
        """Test TemporaryFile()."""

        def test_basic(self):
            tempfile.TemporaryFile()

        def test_has_no_name(self):
            dir = tempfile.mkdtemp()
            f = tempfile.TemporaryFile(dir=dir)
            f.write(b'blat')
            try:
                os.rmdir(dir)
            except:
                f.close()
                os.rmdir(dir)
                raise

        def test_multiple_close(self):
            f = tempfile.TemporaryFile()
            f.write(b'abc\n')
            f.close()
            f.close()
            f.close()

        def test_mode_and_encoding(self):

            def roundtrip(input, *args, **kwargs):
                with tempfile.TemporaryFile(*args, **kwargs) as fileobj:
                    fileobj.write(input)
                    fileobj.seek(0)
                    self.assertEqual(input, fileobj.read())
            roundtrip(b'1234', 'w+b')
            roundtrip('abdc\n', 'w+')
            roundtrip('Λ', 'w+', encoding='utf-16')
            roundtrip('foo\r\n', 'w+', newline='')

        def test_bad_mode(self):
            dir = tempfile.mkdtemp()
            self.addCleanup(os_helper.rmtree, dir)
            with self.assertRaises(ValueError):
                tempfile.TemporaryFile(mode='wr', dir=dir)
            with self.assertRaises(TypeError):
                tempfile.TemporaryFile(mode=2, dir=dir)
            self.assertEqual(os.listdir(dir), [])

        def test_bad_encoding(self):
            dir = tempfile.mkdtemp()
            self.addCleanup(os_helper.rmtree, dir)
            with self.assertRaises(LookupError):
                tempfile.TemporaryFile('w', encoding='bad-encoding', dir=dir)
            self.assertEqual(os.listdir(dir), [])

        def test_unexpected_error(self):
            dir = tempfile.mkdtemp()
            self.addCleanup(os_helper.rmtree, dir)
            with mock.patch('tempfile._O_TMPFILE_WORKS', False), mock.patch('os.unlink') as mock_unlink, mock.patch('os.open') as mock_open, mock.patch('os.close') as mock_close:
                mock_unlink.side_effect = KeyboardInterrupt()
                with self.assertRaises(KeyboardInterrupt):
                    tempfile.TemporaryFile(dir=dir)
            mock_close.assert_called()
            self.assertEqual(os.listdir(dir), [])

class NulledModules:

    def __init__(self, *modules):
        self.refs = [mod.__dict__ for mod in modules]
        self.contents = [ref.copy() for ref in self.refs]

    def __enter__(self):
        for d in self.refs:
            for key in d:
                d[key] = None

    def __exit__(self, *exc_info):
        for d, c in zip(self.refs, self.contents):
            d.clear()
            d.update(c)


# --- test body ---

assert str is tempfile._infer_return_type('')

assert bytes is tempfile._infer_return_type(b'')

assert str is tempfile._infer_return_type(None)
print("TestLowLevelInternals::test_infer_return_type_singles: ok")
