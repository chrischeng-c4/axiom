# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "unicode_file"
# dimension = "behavior"
# case = "test_unicode_files__test_single_files"
# subject = "cpython.test_unicode_file.TestUnicodeFiles.test_single_files"
# kind = "semantic"
# xfail = "auto-ported CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_unicode_file.py"
# status = "filled"
# ///
# mamba-xfail: auto-ported CPython test; mamba promotion pending
# Auto-ported from CPython 3.12 test_unicode_file.py::TestUnicodeFiles::test_single_files
"""Auto-ported test: TestUnicodeFiles::test_single_files (CPython 3.12 oracle)."""


import os, glob, time, shutil
import sys
import unicodedata
import unittest
from test.support.os_helper import rmtree, change_cwd, TESTFN_UNICODE, TESTFN_UNENCODABLE, create_empty_file


if not os.path.supports_unicode_filenames:
    try:
        TESTFN_UNICODE.encode(sys.getfilesystemencoding())
    except (UnicodeError, TypeError):
        raise unittest.SkipTest('No Unicode filesystem semantics on this platform.')

def remove_if_exists(filename):
    if os.path.exists(filename):
        os.unlink(filename)


# --- test body ---
def _do_copyish(filename1, filename2):

    assert os.path.isfile(filename1)
    os.rename(filename1, filename2 + '.new')

    assert not os.path.isfile(filename2)

    assert os.path.isfile(filename1 + '.new')
    os.rename(filename1 + '.new', filename2)

    assert not os.path.isfile(filename1 + '.new')

    assert os.path.isfile(filename2)
    shutil.copy(filename1, filename2 + '.new')
    os.unlink(filename1 + '.new')
    shutil.move(filename1, filename2 + '.new')

    assert not os.path.exists(filename2)

    assert os.path.exists(filename1 + '.new')
    shutil.move(filename1 + '.new', filename2)

    assert not os.path.exists(filename2 + '.new')

    assert os.path.exists(filename1)
    shutil.copy2(filename1, filename2 + '.new')

    assert os.path.isfile(filename1 + '.new')
    os.unlink(filename1 + '.new')

    assert not os.path.exists(filename2 + '.new')

def _do_directory(make_name, chdir_name):
    if os.path.isdir(make_name):
        rmtree(make_name)
    os.mkdir(make_name)
    try:
        with change_cwd(chdir_name):
            cwd_result = os.getcwd()
            name_result = make_name
            cwd_result = unicodedata.normalize('NFD', cwd_result)
            name_result = unicodedata.normalize('NFD', name_result)

            assert os.path.basename(cwd_result) == name_result
    finally:
        os.rmdir(make_name)

def _do_single(filename):

    assert os.path.exists(filename)

    assert os.path.isfile(filename)

    assert os.access(filename, os.R_OK)

    assert os.path.exists(os.path.abspath(filename))

    assert os.path.isfile(os.path.abspath(filename))

    assert os.access(os.path.abspath(filename), os.R_OK)
    os.chmod(filename, 511)
    os.utime(filename, None)
    os.utime(filename, (time.time(), time.time()))
    _do_copyish(filename, filename)

    assert os.path.abspath(filename) == os.path.abspath(glob.glob(glob.escape(filename))[0])
    path, base = os.path.split(os.path.abspath(filename))
    file_list = os.listdir(path)
    base = unicodedata.normalize('NFD', base)
    file_list = [unicodedata.normalize('NFD', f) for f in file_list]

    assert base in file_list

def _test_single(filename):
    remove_if_exists(filename)
    create_empty_file(filename)
    try:
        _do_single(filename)
    finally:
        os.unlink(filename)

    assert not os.path.exists(filename)
    f = os.open(filename, os.O_CREAT | os.O_WRONLY)
    os.close(f)
    try:
        _do_single(filename)
    finally:
        os.unlink(filename)
_test_single(TESTFN_UNICODE)
if TESTFN_UNENCODABLE is not None:
    _test_single(TESTFN_UNENCODABLE)
print("TestUnicodeFiles::test_single_files: ok")
