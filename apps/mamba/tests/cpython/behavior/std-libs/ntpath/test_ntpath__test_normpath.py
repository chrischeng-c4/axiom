# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "ntpath"
# dimension = "behavior"
# case = "test_ntpath__test_normpath"
# subject = "cpython.test_ntpath.TestNtpath.test_normpath"
# kind = "semantic"
# xfail = "auto-ported CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_ntpath.py"
# status = "filled"
# ///
# mamba-xfail: auto-ported CPython test; mamba promotion pending
# Auto-ported from CPython 3.12 test_ntpath.py::TestNtpath::test_normpath
"""Auto-ported test: TestNtpath::test_normpath (CPython 3.12 oracle)."""


import inspect
import ntpath
import os
import string
import subprocess
import sys
import unittest
import warnings
from ntpath import ALLOW_MISSING
from test.support import cpython_only, os_helper
from test.support import TestFailed, is_emscripten
from test.support.os_helper import FakePath
from test import test_genericpath
from tempfile import TemporaryFile


try:
    import nt
except ImportError:
    nt = None

try:
    ntpath._getfinalpathname
except AttributeError:
    HAVE_GETFINALPATHNAME = False
else:
    HAVE_GETFINALPATHNAME = True

try:
    import ctypes
except ImportError:
    HAVE_GETSHORTPATHNAME = False
else:
    HAVE_GETSHORTPATHNAME = True

    def _getshortpathname(path):
        GSPN = ctypes.WinDLL('kernel32', use_last_error=True).GetShortPathNameW
        GSPN.argtypes = [ctypes.c_wchar_p, ctypes.c_wchar_p, ctypes.c_uint32]
        GSPN.restype = ctypes.c_uint32
        result_len = GSPN(path, None, 0)
        if not result_len:
            raise OSError('failed to get short path name 0x{:08X}'.format(ctypes.get_last_error()))
        result = ctypes.create_unicode_buffer(result_len)
        result_len = GSPN(path, result, result_len)
        return result[:result_len]

def _norm(path):
    if isinstance(path, (bytes, str, os.PathLike)):
        return ntpath.normcase(os.fsdecode(path))
    elif hasattr(path, '__iter__'):
        return tuple((ntpath.normcase(os.fsdecode(p)) for p in path))
    return path

def tester(fn, wantResult):
    fn = fn.replace('\\', '\\\\')
    gotResult = eval(fn)
    if wantResult != gotResult and _norm(wantResult) != _norm(gotResult):
        raise TestFailed('%s should return: %s but returned: %s' % (str(fn), str(wantResult), str(gotResult)))
    fn = fn.replace("('", "(b'")
    fn = fn.replace('("', '(b"')
    fn = fn.replace("['", "[b'")
    fn = fn.replace('["', '[b"')
    fn = fn.replace(", '", ", b'")
    fn = fn.replace(', "', ', b"')
    fn = os.fsencode(fn).decode('latin1')
    fn = fn.encode('ascii', 'backslashreplace').decode('ascii')
    with warnings.catch_warnings():
        warnings.simplefilter('ignore', DeprecationWarning)
        gotResult = eval(fn)
    if _norm(wantResult) != _norm(gotResult):
        raise TestFailed('%s should return: %s but returned: %s' % (str(fn), str(wantResult), repr(gotResult)))

def _parameterize(*parameters):
    """Simplistic decorator to parametrize a test

    Runs the decorated test multiple times in subTest, with a value from
    'parameters' passed as an extra positional argument.
    Calls doCleanups() after each run.

    Not for general use. Intended to avoid indenting for easier backports.

    See https://discuss.python.org/t/91827 for discussing generalizations.
    """

    def _parametrize_decorator(func):

        def _parameterized(self, *args, **kwargs):
            for parameter in parameters:
                with self.subTest(parameter):
                    func(self, *args, parameter, **kwargs)
                self.doCleanups()
        return _parameterized
    return _parametrize_decorator

class NtpathTestCase(unittest.TestCase):

    def assertPathEqual(self, path1, path2):
        if path1 == path2 or _norm(path1) == _norm(path2):
            return
        self.assertEqual(path1, path2)

    def assertPathIn(self, path, pathset):
        self.assertIn(_norm(path), _norm(pathset))

class NtCommonTest(test_genericpath.CommonTest, unittest.TestCase):
    pathmodule = ntpath
    attributes = ['relpath']


# --- test body ---
tester("ntpath.normpath('A//////././//.//B')", 'A\\B')
tester("ntpath.normpath('A/./B')", 'A\\B')
tester("ntpath.normpath('A/foo/../B')", 'A\\B')
tester("ntpath.normpath('C:A//B')", 'C:A\\B')
tester("ntpath.normpath('D:A/./B')", 'D:A\\B')
tester("ntpath.normpath('e:A/foo/../B')", 'e:A\\B')
tester("ntpath.normpath('C:///A//B')", 'C:\\A\\B')
tester("ntpath.normpath('D:///A/./B')", 'D:\\A\\B')
tester("ntpath.normpath('e:///A/foo/../B')", 'e:\\A\\B')
tester("ntpath.normpath('..')", '..')
tester("ntpath.normpath('.')", '.')
tester("ntpath.normpath('')", '.')
tester("ntpath.normpath('/')", '\\')
tester("ntpath.normpath('c:/')", 'c:\\')
tester("ntpath.normpath('/../.././..')", '\\')
tester("ntpath.normpath('c:/../../..')", 'c:\\')
tester("ntpath.normpath('../.././..')", '..\\..\\..')
tester("ntpath.normpath('K:../.././..')", 'K:..\\..\\..')
tester("ntpath.normpath('C:////a/b')", 'C:\\a\\b')
tester("ntpath.normpath('//machine/share//a/b')", '\\\\machine\\share\\a\\b')
tester("ntpath.normpath('\\\\.\\NUL')", '\\\\.\\NUL')
tester("ntpath.normpath('\\\\?\\D:/XY\\Z')", '\\\\?\\D:/XY\\Z')
tester("ntpath.normpath('handbook/../../Tests/image.png')", '..\\Tests\\image.png')
tester("ntpath.normpath('handbook/../../../Tests/image.png')", '..\\..\\Tests\\image.png')
tester("ntpath.normpath('handbook///../a/.././../b/c')", '..\\b\\c')
tester("ntpath.normpath('handbook/a/../..///../../b/c')", '..\\..\\b\\c')
tester("ntpath.normpath('//server/share/..')", '\\\\server\\share\\')
tester("ntpath.normpath('//server/share/../')", '\\\\server\\share\\')
tester("ntpath.normpath('//server/share/../..')", '\\\\server\\share\\')
tester("ntpath.normpath('//server/share/../../')", '\\\\server\\share\\')
tester("ntpath.normpath('\\\\foo\\\\')", '\\\\foo\\\\')
tester("ntpath.normpath('\\\\foo\\')", '\\\\foo\\')
tester("ntpath.normpath('\\\\foo')", '\\\\foo')
tester("ntpath.normpath('\\\\')", '\\\\')
print("TestNtpath::test_normpath: ok")
