# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "unicode_file_functions"
# dimension = "behavior"
# case = "unicode_nfkc_file_tests__test_directory"
# subject = "cpython.test_unicode_file_functions.UnicodeNFKCFileTests.test_directory"
# kind = "semantic"
# xfail = "auto-ported CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_unicode_file_functions.py"
# status = "filled"
# ///
# mamba-xfail: auto-ported CPython test; mamba promotion pending
# Auto-ported from CPython 3.12 test_unicode_file_functions.py::UnicodeNFKCFileTests::test_directory
"""Auto-ported test: UnicodeNFKCFileTests::test_directory (CPython 3.12 oracle)."""


import os
import sys
import unittest
import warnings
from unicodedata import normalize
from test.support import os_helper
from test import support


filenames = ['1_abc', '2_ascii', '3_Grüß-Gott', '4_Γειά-σας', '5_Здравствуйте', '6_にぽん', '7_השקצץס', '8_曨曩曫', '9_曨שんдΓß', '10_΅´']

if sys.platform != 'darwin':
    filenames.extend(['11_΅ϓϔ', '12_΅ϓϔ', '13_ ̈́ΎΫ', '14_ẛ῁῍῎῏῝῞῟῭', '15_΅´𣏕', '16_\u2000\u2000\u2000A', '17_\u2001\u2001\u2001A', '18_\u2003\u2003\u2003A', '19_   A'])

if not os.path.supports_unicode_filenames:
    fsencoding = sys.getfilesystemencoding()
    try:
        for name in filenames:
            name.encode(fsencoding)
    except UnicodeEncodeError:
        raise unittest.SkipTest('only NT+ and systems with Unicode-friendly filesystem encoding')


# --- test body ---
files = set(filenames)
normal_form = None
normal_form = 'NFKC'

def _apply_failure(fn, filename, expected_exception=FileNotFoundError, check_filename=True):
    try:
        fn(filename)
        raise AssertionError('expected expected_exception')
    except expected_exception as _aR_e:
        import types as _types_aR
        c = _types_aR.SimpleNamespace(exception=_aR_e)
    exc_filename = c.exception.filename
    if check_filename:

        assert exc_filename == filename

def norm(s):
    if normal_form:
        return normalize(normal_form, s)
    return s
try:
    os.mkdir(os_helper.TESTFN)
except FileExistsError:
    pass
pass
files = set()
for name in files:
    name = os.path.join(os_helper.TESTFN, norm(name))
    with open(name, 'wb') as f:
        f.write((name + '\n').encode('utf-8'))
    os.stat(name)
    files.add(name)
files = files
dirname = os.path.join(os_helper.TESTFN, 'Grüß-曨曩曫')
filename = 'ß-曨曩曫'
with os_helper.temp_cwd(dirname):
    with open(filename, 'wb') as f:
        f.write((filename + '\n').encode('utf-8'))
    os.access(filename, os.R_OK)
    os.remove(filename)
print("UnicodeNFKCFileTests::test_directory: ok")
