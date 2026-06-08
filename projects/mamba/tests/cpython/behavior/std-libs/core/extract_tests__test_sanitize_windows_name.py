# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "core"
# dimension = "behavior"
# case = "extract_tests__test_sanitize_windows_name"
# subject = "cpython.test_core.ExtractTests.test_sanitize_windows_name"
# kind = "semantic"
# xfail = "auto-extracted CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_zipfile/test_core.py"
# status = "filled"
# ///
# mamba-xfail: auto-extracted CPython test; mamba promotion pending
import _pyio
import array
import contextlib
import importlib.util
import io
import itertools
import os
import posixpath
import struct
import subprocess
import sys
import time
import zipfile
from tempfile import TemporaryFile
from random import randint, random, randbytes

def make_test_file():
    with zipfile.ZipFile(TESTFN2, 'w', zipfile.ZIP_STORED) as zipfp:
        for fpath, fdata in SMALL_TEST_DATA:
            zipfp.writestr(fpath, fdata)

def _test_extract_with_target(target):
    make_test_file()
    with zipfile.ZipFile(TESTFN2, 'r') as zipfp:
        for fpath, fdata in SMALL_TEST_DATA:
            writtenfile = zipfp.extract(fpath, target)
            correctfile = os.path.join(target, fpath)
            correctfile = os.path.normpath(correctfile)
            assert os.path.samefile(writtenfile, correctfile)
            with open(writtenfile, 'rb') as f:
                assert fdata.encode() == f.read()
            unlink(writtenfile)
    unlink(TESTFN2)

def _test_extract_all_with_target(target):
    make_test_file()
    with zipfile.ZipFile(TESTFN2, 'r') as zipfp:
        zipfp.extractall(target)
        for fpath, fdata in SMALL_TEST_DATA:
            outfile = os.path.join(target, fpath)
            with open(outfile, 'rb') as f:
                assert fdata.encode() == f.read()
            unlink(outfile)
    unlink(TESTFN2)

def check_file(filename, content):
    assert os.path.isfile(filename)
    with open(filename, 'rb') as f:
        assert f.read() == content

def _test_extract_hackers_arcnames(hacknames):
    for arcname, fixedname in hacknames:
        content = b'foobar' + arcname.encode()
        with zipfile.ZipFile(TESTFN2, 'w', zipfile.ZIP_STORED) as zipfp:
            zinfo = zipfile.ZipInfo()
            zinfo.filename = arcname
            zinfo.external_attr = 384 << 16
            zipfp.writestr(zinfo, content)
        arcname = arcname.replace(os.sep, '/')
        targetpath = os.path.join('target', 'subdir', 'subsub')
        correctfile = os.path.join(targetpath, *fixedname.split('/'))
        with zipfile.ZipFile(TESTFN2, 'r') as zipfp:
            writtenfile = zipfp.extract(arcname, targetpath)
            assert writtenfile == correctfile
        check_file(correctfile, content)
        rmtree('target')
        with zipfile.ZipFile(TESTFN2, 'r') as zipfp:
            zipfp.extractall(targetpath)
        check_file(correctfile, content)
        rmtree('target')
        correctfile = os.path.join(os.getcwd(), *fixedname.split('/'))
        with zipfile.ZipFile(TESTFN2, 'r') as zipfp:
            writtenfile = zipfp.extract(arcname)
            assert writtenfile == correctfile
        check_file(correctfile, content)
        rmtree(fixedname.split('/')[0])
        with zipfile.ZipFile(TESTFN2, 'r') as zipfp:
            zipfp.extractall()
        check_file(correctfile, content)
        rmtree(fixedname.split('/')[0])
        unlink(TESTFN2)
san = zipfile.ZipFile._sanitize_windows_name
assert san(',,?,C:,foo,bar/z', ',') == '_,C_,foo,bar/z'
assert san('a\\b,c<d>e|f"g?h*i', ',') == 'a\\b,c_d_e_f_g_h_i'
assert san('../../foo../../ba..r', '/') == 'foo/ba..r'
assert san('  /  /foo  /  /ba  r', '/') == 'foo/ba  r'
assert san(' . /. /foo ./ . /. ./ba .r', '/') == 'foo/ba .r'

print("ExtractTests::test_sanitize_windows_name: ok")
