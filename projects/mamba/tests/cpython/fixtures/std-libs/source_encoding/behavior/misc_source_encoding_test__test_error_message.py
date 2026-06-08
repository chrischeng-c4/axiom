# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "source_encoding"
# dimension = "behavior"
# case = "misc_source_encoding_test__test_error_message"
# subject = "cpython.test_source_encoding.MiscSourceEncodingTest.test_error_message"
# kind = "semantic"
# xfail = "auto-ported CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_source_encoding.py"
# status = "filled"
# ///
# mamba-xfail: auto-ported CPython test; mamba promotion pending
# Auto-ported from CPython 3.12 test_source_encoding.py::MiscSourceEncodingTest::test_error_message
"""Auto-ported test: MiscSourceEncodingTest::test_error_message (CPython 3.12 oracle)."""


import unittest
from test.support import script_helper, captured_stdout, requires_subprocess, requires_resource
from test.support.os_helper import TESTFN, unlink, rmtree
from test.support.import_helper import unload
import importlib
import os
import sys
import subprocess
import tempfile


# --- test body ---
def verify_bad_module(module_name):

    try:
        __import__('test.tokenizedata.' + module_name)
        raise AssertionError('expected SyntaxError')
    except SyntaxError:
        pass
    path = os.path.dirname(__file__)
    filename = os.path.join(path, 'tokenizedata', module_name + '.py')
    with open(filename, 'rb') as fp:
        bytes = fp.read()

    try:
        compile(bytes, filename, 'exec')
        raise AssertionError('expected SyntaxError')
    except SyntaxError:
        pass
compile(b'# -*- coding: iso-8859-15 -*-\n', 'dummy', 'exec')
compile(b'\xef\xbb\xbf\n', 'dummy', 'exec')
compile(b'\xef\xbb\xbf# -*- coding: utf-8 -*-\n', 'dummy', 'exec')
try:
    compile(b'# -*- coding: fake -*-\n', 'dummy', 'exec')
    raise AssertionError('expected SyntaxError')
except SyntaxError as _aR_e:
    import re as _re_aR
    assert _re_aR.search('fake', str(_aR_e))
try:
    compile(b'\xef\xbb\xbf# -*- coding: iso-8859-15 -*-\n', 'dummy', 'exec')
    raise AssertionError('expected SyntaxError')
except SyntaxError as _aR_e:
    import re as _re_aR
    assert _re_aR.search('iso-8859-15', str(_aR_e))
try:
    compile(b'\xef\xbb\xbf# -*- coding: iso-8859-15 -*-\n', 'dummy', 'exec')
    raise AssertionError('expected SyntaxError')
except SyntaxError as _aR_e:
    import re as _re_aR
    assert _re_aR.search('BOM', str(_aR_e))
try:
    compile(b'\xef\xbb\xbf# -*- coding: fake -*-\n', 'dummy', 'exec')
    raise AssertionError('expected SyntaxError')
except SyntaxError as _aR_e:
    import re as _re_aR
    assert _re_aR.search('fake', str(_aR_e))
try:
    compile(b'\xef\xbb\xbf# -*- coding: fake -*-\n', 'dummy', 'exec')
    raise AssertionError('expected SyntaxError')
except SyntaxError as _aR_e:
    import re as _re_aR
    assert _re_aR.search('BOM', str(_aR_e))
print("MiscSourceEncodingTest::test_error_message: ok")
