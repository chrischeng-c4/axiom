# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "zipimport"
# dimension = "behavior"
# case = "bad_file_zip_import_test_case__test_filename_too_long"
# subject = "cpython.test_zipimport.BadFileZipImportTestCase.testFilenameTooLong"
# kind = "semantic"
# xfail = "auto-ported CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_zipimport.py"
# status = "filled"
# ///
# mamba-xfail: auto-ported CPython test; mamba promotion pending
# Auto-ported from CPython 3.12 test_zipimport.py::BadFileZipImportTestCase::testFilenameTooLong
"""Auto-ported test: BadFileZipImportTestCase::testFilenameTooLong (CPython 3.12 oracle)."""


import sys
import os
import marshal
import importlib
import importlib.util
import struct
import time
import unittest
import unittest.mock
import warnings
from test import support
from test.support import import_helper
from test.support import os_helper
from zipfile import ZipFile, ZipInfo, ZIP_STORED, ZIP_DEFLATED
import zipimport
import linecache
import doctest
import inspect
import io
from traceback import extract_tb, extract_stack, print_tb


try:
    import zlib
except ImportError:
    zlib = None

test_src = 'def get_name():\n    return __name__\ndef get_file():\n    return __file__\n'

test_co = compile(test_src, '<???>', 'exec')

raise_src = 'def do_raise(): raise TypeError\n'

def make_pyc(co, mtime, size):
    data = marshal.dumps(co)
    pyc = importlib.util.MAGIC_NUMBER + struct.pack('<iLL', 0, int(mtime) & 4294967295, size & 4294967295) + data
    return pyc

def module_path_to_dotted_name(path):
    return path.replace(os.sep, '.')

NOW = time.time()

test_pyc = make_pyc(test_co, NOW, len(test_src))

TESTMOD = 'ziptestmodule'

TESTMOD2 = 'ziptestmodule2'

TESTMOD3 = 'ziptestmodule3'

TESTPACK = 'ziptestpackage'

TESTPACK2 = 'ziptestpackage2'

TESTPACK3 = 'ziptestpackage3'

TEMP_DIR = os.path.abspath('junk95142')

TEMP_ZIP = os.path.abspath('junk95142.zip')

pyc_file = importlib.util.cache_from_source(TESTMOD + '.py')

pyc_ext = '.pyc'

def tearDownModule():
    os_helper.unlink(TESTMOD)


# --- test body ---
def _testBogusZipFile():
    os_helper.unlink(TESTMOD)
    fp = open(TESTMOD, 'w+')
    fp.write(struct.pack('=I', 101010256))
    fp.write('a' * 18)
    fp.close()
    z = zipimport.zipimporter(TESTMOD)
    try:
        with warnings.catch_warnings():
            warnings.simplefilter('ignore', DeprecationWarning)

            try:
                z.load_module(None)
                raise AssertionError('expected TypeError')
            except TypeError:
                pass

        try:
            z.find_module(None)
            raise AssertionError('expected TypeError')
        except TypeError:
            pass

        try:
            z.find_spec(None)
            raise AssertionError('expected TypeError')
        except TypeError:
            pass

        try:
            z.exec_module(None)
            raise AssertionError('expected TypeError')
        except TypeError:
            pass

        try:
            z.is_package(None)
            raise AssertionError('expected TypeError')
        except TypeError:
            pass

        try:
            z.get_code(None)
            raise AssertionError('expected TypeError')
        except TypeError:
            pass

        try:
            z.get_data(None)
            raise AssertionError('expected TypeError')
        except TypeError:
            pass

        try:
            z.get_source(None)
            raise AssertionError('expected TypeError')
        except TypeError:
            pass
        error = zipimport.ZipImportError

        assert z.find_spec('abc') is None
        with warnings.catch_warnings():
            warnings.simplefilter('ignore', DeprecationWarning)

            try:
                z.load_module('abc')
                raise AssertionError('expected error')
            except error:
                pass

        try:
            z.get_code('abc')
            raise AssertionError('expected error')
        except error:
            pass

        try:
            z.get_data('abc')
            raise AssertionError('expected OSError')
        except OSError:
            pass

        try:
            z.get_source('abc')
            raise AssertionError('expected error')
        except error:
            pass

        try:
            z.is_package('abc')
            raise AssertionError('expected error')
        except error:
            pass
    finally:
        zipimport._zip_directory_cache.clear()

def assertZipFailure(filename):

    try:
        zipimport.zipimporter(filename)
        raise AssertionError('expected zipimport.ZipImportError')
    except zipimport.ZipImportError:
        pass
assertZipFailure('A' * 33000)
print("BadFileZipImportTestCase::testFilenameTooLong: ok")
